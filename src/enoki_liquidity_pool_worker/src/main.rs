use candid::{candid_method, Principal, Nat};
use ic_cdk_macros::*;

mod main_pool;
mod liquidity;
mod heartbeat;
mod upgrade;

use enoki_exchange_shared::is_managed;
#[allow(unused_imports)]
use enoki_exchange_shared::is_managed::ManagementData;
use enoki_exchange_shared::is_owned;
#[allow(unused_imports)]
use enoki_exchange_shared::is_owned::OwnershipData;
#[allow(unused_imports)]
use enoki_exchange_shared::{has_token_info, types::*};
#[allow(unused_imports)]
use enoki_exchange_shared::has_token_info::AssignedShards;
#[allow(unused_imports)]
use enoki_exchange_shared::interfaces::enoki_wrapped_token::ShardedTransferNotification;

#[init]
#[candid_method(init)]
fn init(owner: Principal, manager: Principal) {
    is_managed::init_manager(ManagementData { manager });
    is_owned::init_owner(OwnershipData {
        owner,
        deploy_time: ic_cdk::api::time(),
    });
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}
