use std::cell::RefCell;

use candid::{candid_method, CandidType, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::{has_token_info, has_trading_fees};
use enoki_exchange_shared::has_token_info::AssignedShards;
use enoki_exchange_shared::is_managed;
use enoki_exchange_shared::types::*;

use crate::other_brokers::init_brokers;

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug)]
pub struct TokenLiquidityData {
    pub liquidity_location: Principal,
    pub assigned_shards_for_worker: AssignedShards,
}

impl Default for TokenLiquidityData {
    fn default() -> Self {
        Self {
            liquidity_location: Principal::anonymous(),
            assigned_shards_for_worker: Default::default(),
        }
    }
}

thread_local! {
    static STATE: RefCell<TokenLiquidityData> = RefCell::new(TokenLiquidityData::default());
}

pub fn get_lp_worker_location() -> Principal {
    STATE.with(|s| s.borrow().liquidity_location)
}

pub fn get_lp_worker_assigned_shard(token: &EnokiToken) -> Principal {
    STATE.with(|s| match token {
        EnokiToken::TokenA => s.borrow().assigned_shards_for_worker.token_a,
        EnokiToken::TokenB => s.borrow().assigned_shards_for_worker.token_b,
    })
}

#[update(name = "initBroker")]
#[candid_method(update, rename = "initBroker")]
async fn init_broker(params: InitBrokerParams) -> AssignedShards {
    let InitBrokerParams {
        other_brokers,
        supply_token_info,
        liquidity_location,
        trading_fees,
    } = params;
    is_managed::assert_is_manager().unwrap();
    init_brokers(other_brokers);
    has_token_info::init_token_info(supply_token_info).await.unwrap();
    let assigned = has_token_info::get_assigned_shards();

    let worker_assigned_shards: Result<(AssignedShards, )> =
        ic_cdk::call(liquidity_location, "getAssignedShards", ())
            .await
            .map_err(|e| e.into_tx_error());
    let worker_assigned_shards = worker_assigned_shards.unwrap().0;

    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.liquidity_location = liquidity_location;
        s.assigned_shards_for_worker = worker_assigned_shards;
    });
    has_trading_fees::init_fee_info(trading_fees);
    assigned
}

pub fn export_stable_storage() -> TokenLiquidityData {
    let data = STATE.with(|s| s.take());
    data
}

pub fn import_stable_storage(data: TokenLiquidityData) {
    STATE.with(|s| s.replace(data));
}
