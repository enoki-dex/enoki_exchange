use std::cell::RefCell;
use std::ops::AddAssign;

use candid::{candid_method, CandidType, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::has_token_info::{finish_init_token_info, start_init_token_info};
use enoki_exchange_shared::is_managed::assert_is_manager;
use enoki_exchange_shared::liquidity::single_user_liquidity_pool::SingleUserLiquidityPool;
use enoki_exchange_shared::types::*;

use crate::worker::{assert_is_worker_contract, get_worker, init_worker_token_data};

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct PooledAmounts {
    worker_pool: SingleUserLiquidityPool,
    added: LiquidityAmount,
    removed: LiquidityAmount,
    traded: LiquidityTrades,
}

thread_local! {
    static STATE: RefCell<PooledAmounts> = RefCell::new(PooledAmounts::default());
}

pub fn export_stable_storage() -> PooledAmounts {
    STATE.with(|b| b.take())
}

pub fn import_stable_storage(data: PooledAmounts) {
    STATE.with(|b| b.replace(data));
}

pub fn lock_liquidity() -> (
    LiquidityAmount,
    LiquidityAmount,
) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.worker_pool.lock_liquidity();
        (
            s.worker_pool.count_locked_add_liquidity(),
            s.worker_pool.count_locked_remove_liquidity(),
        )
    })
}

#[update(name = "initLiquidityPool")]
#[candid_method(update, rename = "initLiquidityPool")]
async fn init_liquidity_pool(supply_token_info: has_token_info::TokenPairInfo) -> Principal {
    assert_is_manager().unwrap();
    let worker = get_worker();
    assert_ne!(worker, Principal::anonymous(), "worker not initialized");
    start_init_token_info(supply_token_info);
    finish_init_token_info().await.unwrap();
    init_worker_token_data().await.unwrap();
    worker
}

#[update(name = "getUpdatedLiquidity")]
#[candid_method(update, rename = "getUpdatedLiquidity")]
fn get_updated_liquidity() -> (
    LiquidityAmount,
    LiquidityAmount,
) {
    assert_is_manager().unwrap();
    lock_liquidity()
}

#[update(name = "resolveLiquidity")]
#[candid_method(update, rename = "resolveLiquidity")]
fn resolve_liquidity(
    added: LiquidityAmount,
    removed: LiquidityAmount,
    traded: LiquidityTrades,
) {
    assert_is_manager().unwrap();
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.added.add_assign(added);
        s.removed.add_assign(removed);
        s.worker_pool.apply_traded(&traded);
        s.traded.add_assign(traded);
    });
}

#[update(name = "updateLiquidity")]
#[candid_method(update, rename = "updateLiquidity")]
fn update_liquidity(
    pending_add: LiquidityAmount,
    pending_remove: LiquidityAmount,
) -> Result<(LiquidityAmount, LiquidityAmount, LiquidityTrades)> {
    assert_is_worker_contract()?;
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let LiquidityAmount {
            token_a: add_a,
            token_b: add_b,
        } = pending_add;
        s.worker_pool.user_add_liquidity(
            TokenAmount {
                token: EnokiToken::TokenA,
                amount: add_a,
            },
        );
        s.worker_pool.user_add_liquidity(
            TokenAmount {
                token: EnokiToken::TokenB,
                amount: add_b,
            },
        );
        s.worker_pool.user_remove_liquidity(pending_remove)?;
        let added = std::mem::take(&mut s.added);
        let removed = std::mem::take(&mut s.removed);
        let traded = std::mem::take(&mut s.traded);
        Ok((added, removed, traded))
    })
}
