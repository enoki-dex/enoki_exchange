use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Nat, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::types::*;

thread_local! {
    static STATE: RefCell<LiquidityState> = RefCell::new(LiquidityState::default());
}

#[derive(serde::Deserialize, serde::Serialize, CandidType, Clone, Debug)]
pub struct LiquidityState {
    pool: Principal,
    broker_liquidity: HashMap<Principal, HashMap<Principal, LiquidityAmount>>,
    proposed_changes: ProposedLiquidityChangesByWorker,
}

impl Default for LiquidityState {
    fn default() -> Self {
        Self {
            pool: Principal::anonymous(),
            broker_liquidity: Default::default(),
            proposed_changes: Default::default()
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, CandidType, Clone, Debug, Default)]
pub struct ProposedLiquidityChangesByWorker {
    to_add: HashMap<Principal, LiquidityAmount>,
    to_remove: HashMap<Principal, LiquidityAmount>,
}

pub fn init_pool(pool: Principal) {
    STATE.with(|s| s.borrow_mut().pool = pool);
}

pub fn add_broker(broker: Principal) {
    STATE.with(|s| s.borrow_mut().broker_liquidity.insert(broker, Default::default()));
}

pub fn get_pool_contract() -> Principal {
    STATE.with(|s| s.borrow().pool)
}

pub async fn get_updated_liquidity_from_pool(
) -> Result<HashMap<Principal, HashMap<Principal, LiquidityAmount>>> {
    let result: Result<(
        HashMap<Principal, LiquidityAmount>,
        HashMap<Principal, LiquidityAmount>,
    )> = ic_cdk::call(get_pool_contract(), "getUpdatedLiquidity", ())
        .await
        .map_err(|e| e.into());
    let (to_add, to_remove) = result?;
    let proposed_changes_by_broker = make_proposed_liquidity_by_broker(&to_add, &to_remove);
    STATE.with(|s| s.borrow_mut().proposed_changes = ProposedLiquidityChangesByWorker {
        to_add,
        to_remove
    });
    Ok(proposed_changes_by_broker)
}

pub async fn update_committed_broker_liquidity(committed: HashMap<Principal, HashMap<Principal, LiquidityAmount>>) -> Result<()>{
    let proposed = STATE.with(|s| std::mem::take(&mut s.borrow_mut().proposed_changes));
    let actual = (proposed.to_add, proposed.to_remove);
    let traded: HashMap<Principal, LiquidityTrades> = Default::default();
    todo!();

    let result: Result<()> = ic_cdk::call(get_pool_contract(), "resolveLiquidity", (actual.0, actual.1, traded))
        .await
        .map_err(|e| e.into());
    result
}

fn make_proposed_liquidity_by_broker(
    proposed_added_liquidity: &HashMap<Principal, LiquidityAmount>,
    proposed_remove_liquidity: &HashMap<Principal, LiquidityAmount>,
) -> HashMap<Principal, HashMap<Principal, LiquidityAmount>> {
    todo!()
}

pub fn export_stable_storage() -> (LiquidityState,) {
    let data = STATE.with(|s| s.take());
    (data,)
}

pub fn import_stable_storage(data: LiquidityState) {
    STATE.with(|s| s.replace(data));
}
