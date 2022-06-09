use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info;
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

#[update(name = "initWorker")]
#[candid_method(update, rename = "initWorker")]
async fn init_worker(
    supply_token_info: has_token_info::TokenPairInfo,
    liquidity_location: Principal,
) -> Result<()> {
    is_managed::assert_is_manager()?;
    has_token_info::init_token_info(supply_token_info).await?;
    STATE.with(|s| s.borrow_mut().liquidity_location = liquidity_location);
    Ok(())
}

pub fn export_stable_storage() -> (TokenLiquidityData,) {
    let data = STATE.with(|s| s.take());
    (data,)
}

pub fn import_stable_storage(data: TokenLiquidityData) {
    STATE.with(|s| s.replace(data));
}
