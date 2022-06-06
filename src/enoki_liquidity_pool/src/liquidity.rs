use std::cell::RefCell;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Deserialize, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::is_managed;
use enoki_exchange_shared::types::*;

use crate::workers::assert_is_worker_contract;

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct PooledAmounts {
    workers: HashMap<Principal, LiquidityAmount>,
    pending_add: HashMap<Principal, LiquidityAmount>,
    pending_remove: HashMap<Principal, LiquidityAmount>,
    pending_add_locked: HashMap<Principal, LiquidityAmount>,
    pending_remove_locked: HashMap<Principal, LiquidityAmount>,
    added: HashMap<Principal, LiquidityAmount>,
    removed: HashMap<Principal, LiquidityAmount>,
}

thread_local! {
    static STATE: RefCell<PooledAmounts> = RefCell::new(PooledAmounts::default());
}

#[update(name = "updateLiquidity")]
#[candid_method(update, rename = "updateLiquidity")]
fn update_liquidity(
    pending_add: LiquidityAmount,
    pending_remove: LiquidityAmount,
) -> Result<(LiquidityAmount, LiquidityAmount)> {
    assert_is_worker_contract()?;
    // return distributions
    todo!()
}
