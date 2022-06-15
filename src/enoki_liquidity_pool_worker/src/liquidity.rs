use std::cell::RefCell;
use std::ops::{AddAssign, Div, Mul};

use candid::{candid_method, CandidType, Nat, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info::{
    get_assigned_shard, get_assigned_shards, AssignedShards,
};
use enoki_exchange_shared::interfaces::enoki_wrapped_token::ShardedTransferNotification;
use enoki_exchange_shared::is_managed::get_manager;
use enoki_exchange_shared::liquidity::liquidity_pool::{LiquidityPool, LiquidityPoolTotalBalance};
use enoki_exchange_shared::types::*;
use enoki_exchange_shared::{has_sharded_users, has_token_info};

thread_local! {
    static STATE: RefCell<LiquidityState> = RefCell::new(LiquidityState::default());
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct LiquidityState {
    locked: bool,
    pool: LiquidityPool,
    earnings_pending: Vec<(Principal, TokenAmount)>,
    rounding_error: LiquidityTrades, //TODO: send these to the accrued fees / use fees to pay for these
}

#[query]
fn get_state() -> LiquidityState {
    STATE.with(|s| s.borrow().clone())
}

pub async fn update_liquidity_with_manager() {
    if STATE.with(|s| {
        let s = s.borrow();
        s.locked
    }) {
        return;
    }
    let (pending_add, pending_remove) = STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.locked = true;
        s.pool.lock_liquidity()
    });
    let response: Result<(LiquidityAmount, LiquidityAmount, LiquidityTrades)> = ic_cdk::call(
        get_manager(),
        "updateLiquidity",
        (pending_add, pending_remove),
    )
    .await
    .map_err(|e| e.into_tx_error());
    let final_result: Result<Vec<(Principal, TokenAmount)>> = match response {
        Ok((added, removed, traded)) => STATE.with(|s| {
            let mut s = s.borrow_mut();
            s.locked = false;
            let rounding_error = apply_traded(traded, &mut s.pool);
            s.rounding_error.add_assign(rounding_error);

            apply_new_liquidity(added, &mut s.pool);
            let withdrawals = calculate_withdrawals(removed, &mut s.pool);
            s.pool.remove_zeros();
            Ok(withdrawals)
        }),
        Err(err) => {
            STATE.with(|s| {
                let mut s = s.borrow_mut();
                s.locked = false;
            });
            Err(err)
        }
    };
    match final_result {
        Ok(withdrawals) => {
            ic_cdk::spawn(distribute_withdrawals(withdrawals));
        }
        Err(error) => {
            ic_cdk::print(format!(
                "error updating liquidity with manager: {:?}",
                error
            ));
        }
    }
}

fn apply_traded(traded: LiquidityTrades, pool: &mut LiquidityPool) -> LiquidityTrades {
    ic_cdk::println!("[worker] resolved: applying traded: {:?}", traded);
    let total = LiquidityPoolTotalBalance::new(pool).get_total_balances();
    ic_cdk::println!("[worker] liquidity before applying traded: {:?}", total);
    let total_a = total.token_a;
    let total_b = total.token_b;
    let balances = pool.get_liquidity_by_principal();

    let changes_per_user = balances
        .into_iter()
        .map(|(&id, balances)| {
            let plus_a;
            let minus_a;
            let plus_b;
            let minus_b;
            if total_a.is_nonzero() {
                minus_a = balances
                    .token_a
                    .clone()
                    .mul(traded.decreased.token_a.clone())
                    .div(total_a.clone());
                plus_b = balances
                    .token_a
                    .clone()
                    .mul(traded.increased.token_b.clone())
                    .div(total_a.clone());
            } else {
                minus_a = Default::default();
                plus_b = Default::default();
            }
            if total_b.is_nonzero() {
                minus_b = balances
                    .token_b
                    .clone()
                    .mul(traded.decreased.token_b.clone())
                    .div(total_b.clone());
                plus_a = balances
                    .token_b
                    .clone()
                    .mul(traded.increased.token_a.clone())
                    .div(total_b.clone());
            } else {
                minus_b = Default::default();
                plus_a = Default::default();
            }
            (
                id,
                LiquidityTrades {
                    increased: LiquidityAmount {
                        token_a: plus_a,
                        token_b: plus_b,
                    },
                    decreased: LiquidityAmount {
                        token_a: minus_a,
                        token_b: minus_b,
                    },
                },
            )
        })
        .collect();

    pool.apply_traded(&changes_per_user);

    let aggr_changes_for_users = changes_per_user.into_iter().map(|(_, val)| val).fold(
        LiquidityTrades::default(),
        |mut sum, next| {
            sum.add_assign(next);
            sum
        },
    );

    ic_cdk::println!(
        "[worker] liquidity after applying traded: {:?}",
        LiquidityPoolTotalBalance::new(pool)
    );

    let mut rounding_error = traded;
    rounding_error.safe_sub_assign(aggr_changes_for_users).unwrap();

    if rounding_error.decreased.token_a.is_nonzero()
        || rounding_error.decreased.token_b.is_nonzero()
        || rounding_error.increased.token_a.is_nonzero()
        || rounding_error.increased.token_b.is_nonzero()
    {
        ic_cdk::println!(
            "[worker] liquidity after applying traded there is a rounding error: {:?}",
            rounding_error
        );
    }

    rounding_error
}

fn apply_new_liquidity(mut amount: LiquidityAmount, pool: &mut LiquidityPool) {
    let mut i = 0;
    ic_cdk::println!(
        "[worker] resolved: adding more total liquidity: {:?}",
        amount
    );
    while amount.token_a.is_nonzero() || amount.token_b.is_nonzero() {
        let item = pool
            .get_locked_add_item(i)
            .expect("inconsistent state between pool and worker");
        let token = item.1.token.clone();
        let amount_left = amount.get_mut(&token);
        if amount_left.is_nonzero() {
            let diff = amount_left.clone().min(item.1.amount.clone());
            amount_left.safe_sub_assign(diff.clone()).unwrap();
            item.1.amount.safe_sub_assign(diff.clone()).unwrap();
            let addr = item.0;
            ic_cdk::println!(
                "[worker] liquidity for user {} was successfully added: {:?} {:?}",
                addr,
                diff,
                token
            );
            pool.get_user_liquidity_mut(addr, &token).add_assign(diff);
        }
        i += 1;
    }
}

fn calculate_withdrawals(
    mut amount: LiquidityAmount,
    pool: &mut LiquidityPool,
) -> Vec<(Principal, TokenAmount)> {
    let mut amounts_to_distribute: Vec<(Principal, TokenAmount)> = Default::default();
    let mut i = 0;
    ic_cdk::println!("[worker] resolved: removing total liquidity: {:?}", amount);
    while amount.token_a.is_nonzero() || amount.token_b.is_nonzero() {
        let item = pool
            .get_locked_remove_item(i)
            .expect("inconsistent state between pool and worker");
        let token = item.1.token.clone();
        let amount_left = amount.get_mut(&token);
        if amount_left.is_nonzero() {
            let addr = item.0;
            let amount_in_lp = pool.get_user_liquidity_mut(addr, &token).clone();
            let item = pool.get_locked_remove_item(i).unwrap();
            item.1.amount = item.1.amount.clone().min(amount_in_lp.clone());
            let diff = amount_left.clone().min(item.1.amount.clone());
            amount_left.safe_sub_assign(diff.clone()).unwrap();
            item.1.amount.safe_sub_assign(diff.clone()).unwrap();
            pool.get_user_liquidity_mut(addr, &token)
                .safe_sub_assign(diff.clone()).unwrap();
            ic_cdk::println!(
                "[worker] liquidity for user {} is successfully being removed: {:?} {:?}",
                addr,
                diff,
                token
            );
            amounts_to_distribute.push((
                addr,
                TokenAmount {
                    token,
                    amount: diff,
                },
            ))
        }

        i += 1;
    }
    amounts_to_distribute
}

async fn distribute_withdrawals(mut withdrawals: Vec<(Principal, TokenAmount)>) {
    let mut past_pending = STATE.with(|s| std::mem::take(&mut s.borrow_mut().earnings_pending));
    withdrawals.append(&mut past_pending);
    let results = futures::future::join_all(
        withdrawals
            .into_iter()
            .map(|(user, withdrawal)| withdraw_for_user(user, withdrawal)),
    )
    .await;
    STATE.with(|s| {
        s.borrow_mut()
            .earnings_pending
            .extend(results.into_iter().filter_map(|failed| failed))
    });
}

async fn withdraw_for_user(
    user: Principal,
    withdrawal: TokenAmount,
) -> Option<(Principal, TokenAmount)> {
    let error;
    match has_sharded_users::get_user_shard(
        user,
        has_token_info::get_token_address(&withdrawal.token),
    ) {
        Ok(user_shard) => {
            let TokenAmount { token, amount } = withdrawal.clone();
            let amount: Nat = amount.into();
            let my_shard = get_assigned_shard(&token);
            ic_cdk::println!("executing shardTransfer to {} with args ({}, {}, {})", my_shard, user_shard, user, amount);
            let result: Result<()> = ic_cdk::call(
                my_shard,
                "shardTransfer",
                (user_shard, user, amount.clone()),
            )
            .await
            .map_err(|e| e.into_tx_error());
            match result {
                Ok(_) => {
                    ic_cdk::println!(
                        "[worker] liquidity for user {} was successfully distributed: {:?} {:?}",
                        user,
                        amount,
                        token
                    );
                    return None;
                }
                Err(err) => {
                    error = err;
                }
            }
        }
        Err(err) => {
            error = err;
        }
    }
    ic_cdk::api::print(format!("failed to remove liquidity: {:?}", error));
    Some((user, withdrawal))
}

#[query(name = "getLiquidity")]
#[candid_method(query, rename = "getLiquidity")]
fn get_liquidity(user: Principal) -> LiquidityAmountNat {
    STATE
        .with(|s| s.borrow().pool.get_user_liquidity(user))
        .unwrap_or_default()
        .into()
}

#[query(name = "getShardsToAddLiquidity")]
#[candid_method(query, rename = "getShardsToAddLiquidity")]
async fn get_shards_to_add_liquidity() -> AssignedShards {
    get_assigned_shards()
}

#[query(name = "isUserRegistered")]
#[candid_method(query, rename = "isUserRegistered")]
pub fn is_user_registered(user: Principal) -> bool {
    has_sharded_users::get_user_shard(user, has_token_info::get_token_address(&EnokiToken::TokenA))
        .is_ok()
        && has_sharded_users::get_user_shard(
            user,
            has_token_info::get_token_address(&EnokiToken::TokenB),
        )
        .is_ok()
}

#[update(name = "register")]
#[candid_method(update)]
async fn register(user: Principal) {
    has_sharded_users::register_user(user).await.unwrap();
}

#[update(name = "addLiquidity")]
#[candid_method(update, rename = "addLiquidity")]
async fn add_liquidity(notification: ShardedTransferNotification) {
    assert_eq!(notification.to, ic_cdk::id());
    let token = has_token_info::parse_from().unwrap();
    let from = notification.from;
    if !is_user_registered(from) {
        panic!(
            "{:?}",
            TxError::UserNotRegistered {
                user: from.to_string(),
                registry: ic_cdk::id().to_string()
            }
        );
    }
    let amount = TokenAmount {
        token,
        amount: notification.value.into(),
    };
    STATE.with(|s| s.borrow_mut().pool.user_add_liquidity(from, amount));
}

#[update(name = "removeLiquidity")]
#[candid_method(update, rename = "removeLiquidity")]
async fn remove_liquidity(amount: LiquidityAmount) {
    let from = ic_cdk::caller();

    STATE
        .with(|s| s.borrow_mut().pool.user_remove_liquidity(from, amount))
        .unwrap();
}

#[update(name = "removeAllLiquidity")]
#[candid_method(update, rename = "removeAllLiquidity")]
async fn remove_all_liquidity() {
    if let Some(liquidity) = STATE.with(|s| s.borrow().pool.get_user_liquidity(ic_cdk::caller())) {
        remove_liquidity(liquidity).await;
    }
}

pub fn export_stable_storage() -> LiquidityState {
    STATE.with(|s| s.take())
}

pub fn import_stable_storage(data: LiquidityState) {
    STATE.with(|s| s.replace(data));
}
