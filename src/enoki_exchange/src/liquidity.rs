use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{AddAssign, SubAssign};

use candid::{candid_method, CandidType, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info::get_token_info;
use enoki_exchange_shared::is_owned::assert_is_owner;
use enoki_exchange_shared::liquidity::{
    RequestForLiquidityChanges, RequestForNewLiquidityTarget, ResponseAboutLiquidityChanges,
};
use enoki_exchange_shared::types::*;

thread_local! {
    static STATE: RefCell<LiquidityState> = RefCell::new(LiquidityState::default());
}

#[derive(serde::Deserialize, serde::Serialize, CandidType, Clone, Debug)]
pub struct LiquidityState {
    pool_address: Principal,
    worker_pool_address: Principal,
    broker_liquidity: HashMap<Principal, LiquidityAmount>,
    excess_liquidity: LiquidityAmount,
    lp_proposed_changes: RequestForLiquidityChanges,
}

impl Default for LiquidityState {
    fn default() -> Self {
        Self {
            pool_address: Principal::anonymous(),
            worker_pool_address: Principal::anonymous(),
            broker_liquidity: Default::default(),
            excess_liquidity: Default::default(),
            lp_proposed_changes: Default::default(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, CandidType, Clone, Debug, Default)]
pub struct ProposedLiquidityChanges {
    to_add: LiquidityAmount,
    to_remove: LiquidityAmount,
}

#[update(name = "initPool")]
#[candid_method(update, rename = "initPool")]
async fn init_pool(pool: Principal) {
    assert_is_owner().unwrap();
    let response: Result<(Principal, )> =
        ic_cdk::call(pool, "initLiquidityPool", (get_token_info(), ))
            .await
            .map_err(|e| e.into_tx_error());
    let worker = response.unwrap().0;
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.pool_address = pool;
        s.worker_pool_address = worker;
    });
}

pub fn init_broker_lp(broker: Principal) {
    STATE.with(|s| {
        s.borrow_mut()
            .broker_liquidity
            .insert(broker, Default::default())
    });
}

pub fn get_pool_contract() -> Principal {
    STATE.with(|s| s.borrow().pool_address)
}

#[query(name = "getLiquidityLocation")]
#[candid_method(update, rename = "getLiquidityLocation")]
pub fn get_liquidity_location() -> Principal {
    let location = STATE.with(|s| s.borrow().worker_pool_address);
    assert_ne!(
        location,
        Principal::anonymous(),
        "LP has not yet been initialized"
    );
    location
}

pub async fn get_updated_liquidity_from_pool() -> Result<RequestForNewLiquidityTarget> {
    let result: Result<(LiquidityAmount, LiquidityAmount)> =
        ic_cdk::call(get_pool_contract(), "getUpdatedLiquidity", ())
            .await
            .map_err(|e| e.into_tx_error());
    let (to_add, to_remove) = result?;
    let request_from_pool = RequestForLiquidityChanges { to_add, to_remove };
    let proposed_target_for_brokers =
        create_requests_for_broker_liquidity_targets(&request_from_pool);
    STATE.with(|s| s.borrow_mut().lp_proposed_changes = request_from_pool);
    Ok(proposed_target_for_brokers)
}

fn create_requests_for_broker_liquidity_targets(
    request_from_pool: &RequestForLiquidityChanges,
) -> RequestForNewLiquidityTarget {
    let current_liquidity = STATE.with(|s| s.borrow().broker_liquidity.clone());
    let broker_count = current_liquidity.len();

    let mut liquidity_reserves = request_from_pool.to_add.clone();
    liquidity_reserves.add_assign(STATE.with(|s| s.borrow().excess_liquidity.clone()));
    let max_currently_available_per_broker = liquidity_reserves.clone().div_int(broker_count);

    let mut total_liquidity_target: LiquidityAmount = current_liquidity
        .into_iter()
        .map(|(_broker, current_amount)| current_amount)
        .fold(LiquidityAmount::default(), |mut sum, next| {
            sum.add_assign(next);
            sum
        });
    total_liquidity_target.add_assign(liquidity_reserves);
    total_liquidity_target.sub_assign_or_zero(request_from_pool.to_remove.clone());

    let target_liquidity_per_broker = total_liquidity_target.clone().div_int(broker_count);

    RequestForNewLiquidityTarget {
        target: target_liquidity_per_broker,
        extra_liquidity_available: max_currently_available_per_broker,
    }
}

pub async fn update_committed_broker_liquidity(
    response: HashMap<Principal, ResponseAboutLiquidityChanges>,
) -> Result<()> {
    apply_changes(&response);
    let (mut added, mut removed, traded): (LiquidityAmount, LiquidityAmount, LiquidityTrades) =
        response.into_iter().fold(
            (Default::default(), Default::default(), Default::default()),
            |(mut added, mut removed, mut traded), (_, changes)| {
                added.add_assign(changes.added);
                removed.add_assign(changes.removed);
                traded.add_assign(changes.traded);
                (added, removed, traded)
            },
        );

    let proposed_by_lp = STATE.with(|s| std::mem::take(&mut s.borrow_mut().lp_proposed_changes));
    let excess_added = added.sub_or_zero(&proposed_by_lp.to_add);
    added.sub_assign(excess_added.clone());
    let excess_removed = removed.sub_or_zero(&proposed_by_lp.to_remove);
    removed.sub_assign(excess_removed.clone());
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.excess_liquidity.add_assign(excess_added);
        s.excess_liquidity.sub_assign(excess_removed);
    });

    let result: Result<()> = ic_cdk::call(
        get_pool_contract(),
        "resolveLiquidity",
        (added, removed, traded),
    )
        .await
        .map_err(|e| e.into_tx_error());
    result
}

fn apply_changes(changes: &HashMap<Principal, ResponseAboutLiquidityChanges>) {
    STATE.with(|s| {
        for (broker_id, liquidity) in s.borrow_mut().broker_liquidity.iter_mut() {
            if let Some(changes) = changes.get(broker_id) {
                liquidity.add_assign(changes.traded.increased.clone());
                liquidity.sub_assign(changes.traded.decreased.clone());
                liquidity.add_assign(changes.added.clone());
                liquidity.sub_assign(changes.removed.clone());
            }
        }
    })
}

pub fn export_stable_storage() -> LiquidityState {
    STATE.with(|s| s.take())
}

pub fn import_stable_storage(data: LiquidityState) {
    STATE.with(|s| s.replace(data));
}
