use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::has_token_info::AssignedShards;
use enoki_exchange_shared::is_managed;
use enoki_exchange_shared::types::*;

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug)]
pub struct TokenLiquidityData {
    pub liquidity_location: Principal,
}

impl Default for TokenLiquidityData {
    fn default() -> Self {
        Self {
            liquidity_location: Principal::anonymous(),
        }
    }
}

thread_local! {
    static STATE: RefCell<TokenLiquidityData> = RefCell::new(TokenLiquidityData::default());
}

pub fn get_lp_worker_location() -> Principal {
    STATE.with(|s| s.borrow().liquidity_location)
}

#[update(name = "initBroker")]
#[candid_method(update, rename = "initBroker")]
async fn init_broker(
    supply_token_info: has_token_info::TokenPairInfo,
    liquidity_location: Principal,
) -> Result<AssignedShards> {
    is_managed::assert_is_manager()?;
    has_token_info::init_token_info(supply_token_info).await?;
    let assigned = has_token_info::get_assigned_shards();
    STATE.with(|s| s.borrow_mut().liquidity_location = liquidity_location);
    Ok(assigned)
}

pub fn export_stable_storage() -> (TokenLiquidityData,) {
    let data = STATE.with(|s| s.take());
    (data,)
}

pub fn import_stable_storage(data: TokenLiquidityData) {
    STATE.with(|s| s.replace(data));
}
