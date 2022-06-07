use candid::{candid_method, types::number::Nat, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info::{self, AssignedShards, TokenInfo};
use enoki_exchange_shared::is_owned::{self, OwnershipData};

#[allow(unused_imports)]
use crate::brokers::BrokerState;

#[allow(unused_imports)]
use enoki_exchange_shared::types::Result;

mod brokers;
mod heartbeat;
mod orders;
mod synchronize;

#[init]
#[candid_method(init)]
async fn init(owner: Principal, token_a: Principal, token_b: Principal) {
    is_owned::init_owner(OwnershipData {
        owner,
        deploy_time: ic_cdk::api::time(),
    });
    let token_info = TokenInfo { token_a, token_b };
    has_token_info::init_token_info(token_info).await.unwrap();
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}
