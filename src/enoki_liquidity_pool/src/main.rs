#[allow(unused_imports)]
use std::collections::HashMap;

use candid::{candid_method, Nat, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info::{AssignedShards, TokenPairInfo};
use enoki_exchange_shared::is_owned;
#[allow(unused_imports)]
use enoki_exchange_shared::is_owned::OwnershipData;
#[allow(unused_imports)]
use enoki_exchange_shared::{has_token_info, has_token_info::TokenInfo, types::*};
#[allow(unused_imports)]
use workers::WorkerContractData;

mod exchange;
mod liquidity;
mod workers;

#[init]
#[candid_method(init)]
async fn init(owner: Principal, exchange: Principal, token_a: Principal, token_b: Principal) {
    is_owned::init_owner(OwnershipData {
        owner,
        deploy_time: ic_cdk::api::time(),
    });
    exchange::init_exchange_information(exchange);
    let token_info = TokenPairInfo {
        token_a: TokenInfo {
            principal: token_a,
            units_per_lot: Default::default(),
            min_price_interval_lots: 0,
        },
        token_b: TokenInfo {
            principal: token_b,
            units_per_lot: Default::default(),
            min_price_interval_lots: 0,
        },
    };
    has_token_info::init_token_info(token_info).await.unwrap();
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}
