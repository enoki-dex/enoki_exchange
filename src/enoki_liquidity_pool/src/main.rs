#[allow(unused_imports)]
use std::collections::HashMap;

use candid::{candid_method, Principal};
use ic_cdk_macros::*;

#[allow(unused_imports)]
use enoki_exchange_shared::is_managed::ManagementData;
#[allow(unused_imports)]
use enoki_exchange_shared::is_owned::OwnershipData;
#[allow(unused_imports)]
use enoki_exchange_shared::{
    has_token_info,
    has_token_info::{AssignedShards, TokenPairInfo},
    has_trading_fees::TradingFees,
    types::*,
};
use enoki_exchange_shared::{is_managed, is_owned};
#[allow(unused_imports)]
use worker::WorkerContractData;

mod liquidity;
mod shared_candid_methods;
mod upgrade;
mod worker;

#[init]
#[candid_method(init)]
fn init() {
    is_owned::init_owner(OwnershipData {
        owner: ic_cdk::caller(),
        deploy_time: ic_cdk::api::time(),
    });
}

#[update(name = "finishInit")]
#[candid_method(update, rename = "finishInit")]
fn finish_init(exchange: Principal) {
    is_owned::assert_is_owner().unwrap();
    assert_eq!(is_managed::get_manager(), Principal::anonymous(), "already init");
    is_managed::init_manager(ManagementData { manager: exchange });
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}
