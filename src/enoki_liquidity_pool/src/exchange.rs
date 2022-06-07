use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Deserialize, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::liquidity::liquidity_pool::LiquidityPool;
use enoki_exchange_shared::types::*;
use enoki_exchange_shared::is_managed::{self, ManagementData};
use crate::liquidity::lock_liquidity;

pub fn assert_is_exchange() -> Result<()> {
    is_managed::assert_is_manager()
}

pub fn init_exchange_information(exchange: Principal) {
    is_managed::init_manager(ManagementData { manager: exchange })
}

pub fn export_stable_storage() -> (ManagementData,) {
    is_managed::export_stable_storage()
}

pub fn import_stable_storage(data: ManagementData) {
    is_managed::import_stable_storage(data);
}