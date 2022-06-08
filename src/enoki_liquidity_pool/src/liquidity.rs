use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::AddAssign;

use candid::{candid_method, CandidType, Deserialize, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::is_managed;
use enoki_exchange_shared::liquidity::liquidity_pool::LiquidityPool;
use enoki_exchange_shared::types::*;

use crate::exchange::assert_is_exchange;
use crate::workers::assert_is_worker_contract;

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct PooledAmounts {
    workers_pool: LiquidityPool,
    added: HashMap<Principal, LiquidityAmount>,
    removed: HashMap<Principal, LiquidityAmount>,
    traded: HashMap<Principal, LiquidityTrades>,
}

thread_local! {
    static STATE: RefCell<PooledAmounts> = RefCell::new(PooledAmounts::default());
}

pub fn export_stable_storage() -> (PooledAmounts,) {
    let data: PooledAmounts = STATE.with(|b| b.take());
    (data,)
}

pub fn import_stable_storage(data: PooledAmounts) {
    STATE.with(|b| b.replace(data));
}

pub fn lock_liquidity() -> (
    HashMap<Principal, LiquidityAmount>,
    HashMap<Principal, LiquidityAmount>,
) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.workers_pool.lock_liquidity();
        (
            s.workers_pool.count_locked_add_liquidity_by_principal(),
            s.workers_pool.count_locked_remove_liquidity_by_principal(),
        )
    })
}

#[update(name = "getUpdatedLiquidity")]
#[candid_method(update, rename = "getUpdatedLiquidity")]
fn get_updated_liquidity() -> (
    HashMap<Principal, LiquidityAmount>,
    HashMap<Principal, LiquidityAmount>,
) {
    assert_is_exchange().unwrap();
    lock_liquidity()
}

#[update(name = "resolveLiquidity")]
#[candid_method(update, rename = "resolveLiquidity")]
fn resolve_liquidity(
    added: HashMap<Principal, LiquidityAmount>,
    removed: HashMap<Principal, LiquidityAmount>,
    traded: HashMap<Principal, LiquidityTrades>,
) {
    assert_is_exchange().unwrap();
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        for (worker, a) in added {
            s.added.entry(worker).or_default().add_assign(a);
        }
        for (worker, r) in removed {
            s.removed.entry(worker).or_default().add_assign(r);
        }
        s.workers_pool.apply_traded(&traded);
        for (worker, traded) in traded {
            s.traded.entry(worker).or_default().add_assign(traded);
        }
    });
}

#[update(name = "updateLiquidity")]
#[candid_method(update, rename = "updateLiquidity")]
fn update_liquidity(
    pending_add: LiquidityAmount,
    pending_remove: LiquidityAmount,
) -> Result<(LiquidityAmount, LiquidityAmount, LiquidityTrades)> {
    assert_is_worker_contract()?;
    let worker = ic_cdk::caller();
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let LiquidityAmount {
            token_a: add_a,
            token_b: add_b,
        } = pending_add;
        s.workers_pool.user_add_liquidity(
            worker,
            TokenAmount {
                token: EnokiToken::TokenA,
                amount: add_a,
            },
        );
        s.workers_pool.user_add_liquidity(
            worker,
            TokenAmount {
                token: EnokiToken::TokenB,
                amount: add_b,
            },
        );
        s.workers_pool
            .user_remove_liquidity(worker, pending_remove)?;
        let added = std::mem::take(s.added.entry(worker).or_default());
        let removed = std::mem::take(s.removed.entry(worker).or_default());
        let rewards = std::mem::take(s.traded.entry(worker).or_default());
        Ok((added, removed, rewards))
    })
}
