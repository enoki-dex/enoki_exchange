use std::borrow::BorrowMut;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::ops::{AddAssign, Div, Mul, Sub, SubAssign};

use candid::{candid_method, CandidType, Deserialize, Nat, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_sharded_users::{get_user_shard, register_user};
use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::has_token_info::{
    get_assigned_shard, get_assigned_shards, AssignedShards,
};
use enoki_exchange_shared::interfaces::enoki_wrapped_token::ShardedTransferNotification;
use enoki_exchange_shared::is_managed;
use enoki_exchange_shared::is_managed::get_manager;
use enoki_exchange_shared::liquidity::liquidity_pool::LiquidityPool;
use enoki_exchange_shared::types::*;

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

#[query(name = "getLiquidity")]
#[candid_method(query, rename = "getLiquidity")]
fn get_liquidity(user: Principal) -> LiquidityAmount {
    STATE
        .with(|s| s.borrow().pool.get_user_liquidity(user))
        .unwrap_or_default()
}

pub async fn update_liquidity_with_manager() {
    if STATE.with(|s| {
        let s = s.borrow();
        s.locked || (s.pool.nothing_pending())
    }) {
        return;
    }
    let (pending_add, pending_remove) = STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.locked = true;
        s.pool.lock_liquidity();
        (
            s.pool.count_locked_add_liquidity(),
            s.pool.count_locked_remove_liquidity(),
        )
    });
    let response: Result<(Result<(LiquidityAmount, LiquidityAmount, LiquidityTrades)>,)> =
        ic_cdk::call(
            get_manager(),
            "updateLiquidity",
            (pending_add, pending_remove),
        )
        .await
        .map_err(|e| e.into());
    let final_result: Result<Vec<(Principal, TokenAmount)>> = match response {
        Ok((Ok((added, removed, traded)),)) => STATE.with(|s| {
            let mut s = s.borrow_mut();
            s.locked = false;
            apply_traded(traded, &mut s.pool);
            apply_new_liquidity(added, &mut s.pool);
            let withdrawals = calculate_withdrawals(removed, &mut s.pool);
            s.pool.remove_zeros();
            Ok(withdrawals)
        }),
        Ok((Err(err),)) | Err(err) => {
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

fn apply_traded(traded: LiquidityTrades, pool: &mut LiquidityPool) {
    let balances = pool.get_liquidity_by_principal();
    let total_a: StableNat = balances
        .values()
        .map(|val| val.get(&EnokiToken::TokenA).clone())
        .sum();
    let total_b: StableNat = balances
        .values()
        .map(|val| val.get(&EnokiToken::TokenA).clone())
        .sum();

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

    let mut rounding_error = traded;
    rounding_error.sub_assign(aggr_changes_for_users);
    STATE.with(|s| {
        s.borrow_mut().rounding_error.add_assign(rounding_error);
    });
}

fn apply_new_liquidity(mut amount: LiquidityAmount, pool: &mut LiquidityPool) {
    let mut i = 0;
    while amount.token_a.is_nonzero() || amount.token_b.is_nonzero() {
        let item = pool
            .get_locked_add_item(i)
            .expect("inconsistent state between pool and worker");
        let token = item.1.token.clone();
        let amount_left = amount.get_mut(&token);
        if amount_left.is_nonzero() {
            let diff = amount_left.clone().min(item.1.amount.clone());
            amount_left.sub_assign(diff.clone());
            item.1.amount.sub_assign(diff.clone());
            let addr = item.0;
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
            amount_left.sub_assign(diff.clone());
            item.1.amount.sub_assign(diff.clone());
            pool.get_user_liquidity_mut(addr, &token)
                .sub_assign(diff.clone());
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
    let user_shard = get_user_shard(user, has_token_info::get_token_address(&withdrawal.token));
    let TokenAmount { token, amount } = withdrawal.clone();
    let amount: Nat = amount.into();
    let my_shard = get_assigned_shard(&token);
    let result: Result<()> = ic_cdk::call(my_shard, "shardTransfer", (user_shard, user, amount))
        .await
        .map_err(|e| e.into());
    match result {
        Ok(_) => None,
        Err(err) => {
            ic_cdk::api::print(format!("failed to remove liquidity: {:?}", err));
            Some((user, withdrawal))
        }
    }
}

#[query(name = "getShardsToAddLiquidity")]
#[candid_method(query, rename = "getShardsToAddLiquidity")]
async fn get_shards_to_add_liquidity() -> AssignedShards {
    get_assigned_shards()
}

#[update(name = "addLiquidity")]
#[candid_method(update, rename = "addLiquidity")]
async fn add_liquidity(notification: ShardedTransferNotification) -> Result<()> {
    assert_eq!(notification.to, ic_cdk::id());
    let token = has_token_info::parse_from()?;
    let from = notification.from;
    register_user(
        from,
        has_token_info::get_token_address(&token),
        notification.from_shard,
    );
    let amount = TokenAmount {
        token,
        amount: notification.value.into(),
    };
    STATE.with(|s| s.borrow_mut().pool.user_add_liquidity(from, amount));
    Ok(())
}

#[update(name = "removeLiquidity")]
#[candid_method(update, rename = "removeLiquidity")]
async fn remove_liquidity(amount: LiquidityAmount) -> Result<()> {
    let from = ic_cdk::caller();

    STATE.with(|s| s.borrow_mut().pool.user_remove_liquidity(from, amount))
}

pub fn export_stable_storage() -> LiquidityState {
    STATE.with(|s| s.take())
}

pub fn import_stable_storage(data: LiquidityState) {
    STATE.with(|s| s.replace(data));
}