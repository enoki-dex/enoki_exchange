use candid::{candid_method, Principal};
#[allow(unused_imports)]
use candid::Nat;
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info::{self, TokenInfo, TokenPairInfo};
#[allow(unused_imports)]
use enoki_exchange_shared::has_token_info::AssignedShards;
#[allow(unused_imports)]
use enoki_exchange_shared::has_trading_fees::TradingFees;
use enoki_exchange_shared::is_owned::{self, assert_is_owner, OwnershipData};
#[allow(unused_imports)]
use enoki_exchange_shared::types::Result;

#[allow(unused_imports)]
use crate::brokers::BrokerState;

mod brokers;
mod heartbeat;
mod liquidity;
mod orders;
mod synchronize;
mod upgrade;
mod shared_candid_methods;

#[init]
#[candid_method(init)]
fn init(owner: Principal, token_a: Principal, token_b: Principal, price_number_of_decimals: u64) {
    is_owned::init_owner(OwnershipData {
        owner,
        deploy_time: ic_cdk::api::time(),
    });
    let token_info = TokenPairInfo {
        token_a: TokenInfo { principal: token_a },
        token_b: TokenInfo { principal: token_b },
        price_number_of_decimals,
    };
    has_token_info::start_init_token_info(token_info);
}

#[update(name = "finishInit")]
#[candid_method(update, rename = "finishInit")]
async fn finish_init() {
    assert_is_owner().unwrap();
    has_token_info::finish_init_token_info().await.unwrap();
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}
