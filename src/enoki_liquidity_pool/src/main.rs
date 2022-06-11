#[allow(unused_imports)]
use std::collections::HashMap;

use candid::{candid_method, Nat, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info::{AssignedShards, TokenPairInfo};
use enoki_exchange_shared::{is_managed, is_owned};
#[allow(unused_imports)]
use enoki_exchange_shared::is_owned::OwnershipData;
#[allow(unused_imports)]
use enoki_exchange_shared::is_managed::ManagementData;
#[allow(unused_imports)]
use enoki_exchange_shared::{has_token_info, has_token_info::TokenInfo, types::*};
#[allow(unused_imports)]
use worker::WorkerContractData;

mod liquidity;
mod worker;
mod upgrade;

#[init]
#[candid_method(init)]
async fn init(owner: Principal, exchange: Principal, token_a: Principal, token_b: Principal) {
    is_owned::init_owner(OwnershipData {
        owner,
        deploy_time: ic_cdk::api::time(),
    });
    is_managed::init_manager(ManagementData {
        manager: exchange
    });
    let token_info = TokenPairInfo {
        token_a: TokenInfo {
            principal: token_a,
        },
        token_b: TokenInfo {
            principal: token_b,
        },
        price_number_of_decimals: 0, //TODO: are these necessary on the LP side?
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
