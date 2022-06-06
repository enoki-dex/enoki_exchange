use std::cell::RefCell;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Deserialize, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::is_managed;
use enoki_exchange_shared::types::*;

use crate::workers::assert_is_worker_contract;

#[derive(serde::Serialize, serde::Deserialize, Hash, CandidType, Clone, Debug, Eq, PartialEq)]
struct WorkerAndUser {
    worker: Principal,
    user: Principal,
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct PooledAmounts {
    owners: HashMap<WorkerAndUser, TokenAmount>,
    pending_add: HashMap<WorkerAndUser, TokenAmount>,
    pending_remove: HashMap<WorkerAndUser, TokenAmount>,
}

thread_local! {
    static STATE: RefCell<PooledAmounts> = RefCell::new(PooledAmounts::default());
}

#[query(name = "getLiquidity")]
#[candid_method(query, rename = "getLiquidity")]
fn get_liquidity(worker: Principal, user: Principal) -> Option<TokenAmount> {
    STATE.with(|s| {
        s.borrow()
            .owners
            .get(&WorkerAndUser { worker, user })
            .cloned()
    })
}

#[update(name = "updateLiquidity")]
#[candid_method(update, rename = "updateLiquidity")]
fn update_liquidity(
    pending_add: LiquidityAmount,
    pending_remove: LiquidityAmount,
) -> Result<Vec<(Principal, TokenAmount)>> {
    assert_is_worker_contract().unwrap();
    // return distributions
    todo!()
}
