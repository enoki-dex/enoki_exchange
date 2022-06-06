use std::borrow::BorrowMut;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::ops::{AddAssign, SubAssign};

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
use enoki_exchange_shared::types::*;

use crate::liquidity::pool::Pool;

mod pool;

thread_local! {
    static STATE: RefCell<LiquidityState> = RefCell::new(LiquidityState::default());
}

#[derive(Deserialize, CandidType, Clone, Debug, Default)]
struct LiquidityState {
    locked: bool,
    pool: Pool,
    earnings_pending: Vec<(Principal, TokenAmount)>,
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
    let response: Result<(Result<(LiquidityAmount, LiquidityAmount)>,)> = ic_cdk::call(
        get_manager(),
        "updateLiquidity",
        (pending_add, pending_remove),
    )
    .await
    .map_err(|e| e.into());
    let final_result: Result<Vec<(Principal, TokenAmount)>> = match response {
        Ok((Ok((added, removed)),)) => STATE.with(|s| {
            let mut s = s.borrow_mut();
            s.locked = false;
            apply_new_liquidity(added, &mut s.pool);
            let earnings = calculate_earnings(removed, &mut s.pool);
            s.pool.remove_zeros();
            Ok(earnings)
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
        Ok(earnings) => {
            ic_cdk::spawn(distribute_earnings(earnings));
        }
        Err(error) => {
            ic_cdk::print(format!(
                "error updating liquidity with manager: {:?}",
                error
            ));
        }
    }
}

fn apply_new_liquidity(mut amount: LiquidityAmount, pool: &mut Pool) {
    let mut i = 0;
    while amount.token_a.is_nonzero() || amount.token_b.is_nonzero() {
        let item = pool
            .get_locked_add_item(i)
            .expect("inconsistent state between pool and worker");
        let token = item.1.token.clone();
        let amount_left = amount.get_mut(&token);
        if amount_left.is_nonzero() {
            let diff = amount_left.min(&item.1.amount);
            amount_left.sub_assign(diff.clone());
            item.1.amount.sub_assign(diff.clone());
            let addr = item.0;
            pool.get_user_liquidity_mut(addr, &token).add_assign(diff);
        }
        i += 1;
    }
}

fn calculate_earnings(
    mut amount: LiquidityAmount,
    pool: &mut Pool,
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
            item.1.amount = item.1.amount.min(&amount_in_lp);
            let diff = amount_left.min(&item.1.amount);
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

async fn distribute_earnings(mut earnings: Vec<(Principal, TokenAmount)>) {
    let mut past_pending = STATE.with(|s| std::mem::take(&mut s.borrow_mut().earnings_pending));
    earnings.append(&mut past_pending);
    let results = futures::future::join_all(
        earnings
            .into_iter()
            .map(|(user, earning)| distribute_earnings_to_user(user, earning)),
    )
    .await;
    STATE.with(|s| {
        s.borrow_mut()
            .earnings_pending
            .extend(results.into_iter().filter_map(|failed| failed))
    });
}

async fn distribute_earnings_to_user(
    user: Principal,
    earnings: TokenAmount,
) -> Option<(Principal, TokenAmount)> {
    let user_shard = get_user_shard(user);
    let TokenAmount { token, amount } = earnings.clone();
    let amount: Nat = amount.into();
    let my_shard = get_assigned_shard(&token);
    let result: Result<()> = ic_cdk::call(my_shard, "shardTransfer", (user_shard, user, amount))
        .await
        .map_err(|e| e.into());
    match result {
        Ok(_) => None,
        Err(err) => {
            ic_cdk::api::print(format!("failed to remove liquidity: {:?}", err));
            Some((user, earnings))
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
    let amount = TokenAmount {
        token,
        amount: notification.value.into(),
    };
    let from = notification.from;
    register_user(ShardedPrincipal {
        shard: notification.from_shard,
        principal: from,
    });
    STATE.with(|s| s.borrow_mut().pool.user_add_liquidity(from, amount));
    Ok(())
}

#[update(name = "removeLiquidity")]
#[candid_method(update, rename = "removeLiquidity")]
async fn remove_liquidity(amount: LiquidityAmount) -> Result<()> {
    let from = ic_cdk::caller();

    STATE.with(|s| s.borrow_mut().pool.user_remove_liquidity(from, amount))
}
