extern crate core;

use candid::{candid_method, types::number::Nat, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info::{self, AssignedShards, TokenInfo, TokenPairInfo};
use enoki_exchange_shared::is_managed::{self, ManagementData};
use enoki_exchange_shared::is_owned::{self, OwnershipData};
#[allow(unused_imports)]
use enoki_exchange_shared::types::*;
#[allow(unused_imports)]
use enoki_exchange_shared::liquidity::*;
#[allow(unused_imports)]
use enoki_exchange_shared::interfaces::enoki_wrapped_token::ShardedTransferNotification;
#[allow(unused_imports)]
use enoki_exchange_shared::has_trading_fees::TradingFees;
#[allow(unused_imports)]
use payoffs::PendingTransfer;

mod token_liquidity_params;
mod orders;
mod liquidity;
mod payoffs;
mod other_brokers;
mod upgrade;

#[init]
#[candid_method(init)]
async fn init(owner: Principal, exchange: Principal) {
    is_owned::init_owner(OwnershipData {
        owner,
        deploy_time: ic_cdk::api::time(),
    });
    is_managed::init_manager(ManagementData { manager: exchange });
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}
