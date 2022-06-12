extern crate core;

use candid::{candid_method, Principal};
use ic_cdk_macros::*;

#[allow(unused_imports)]
use enoki_exchange_shared::has_token_info::{self, AssignedShards, TokenPairInfo};
#[allow(unused_imports)]
use enoki_exchange_shared::has_trading_fees::TradingFees;
#[allow(unused_imports)]
use enoki_exchange_shared::interfaces::enoki_wrapped_token::ShardedTransferNotification;
use enoki_exchange_shared::is_managed::{self, ManagementData};
use enoki_exchange_shared::is_owned::{self, assert_is_owner, OwnershipData};
#[allow(unused_imports)]
use enoki_exchange_shared::liquidity::*;
#[allow(unused_imports)]
use enoki_exchange_shared::types::*;
#[allow(unused_imports)]
use payoffs::PendingTransfer;

mod token_liquidity_params;
mod orders;
mod liquidity;
mod payoffs;
mod other_brokers;
mod upgrade;
mod shared_candid_methods;

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
    assert_is_owner().unwrap();
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
