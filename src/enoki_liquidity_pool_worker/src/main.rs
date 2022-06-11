use candid::{candid_method, Principal};
use ic_cdk_macros::*;

#[allow(unused_imports)]
use enoki_exchange_shared::{has_token_info, types::*};
#[allow(unused_imports)]
use enoki_exchange_shared::has_token_info::{AssignedShards, TokenPairInfo};
#[allow(unused_imports)]
use enoki_exchange_shared::interfaces::enoki_wrapped_token::ShardedTransferNotification;
use enoki_exchange_shared::is_managed;
#[allow(unused_imports)]
use enoki_exchange_shared::is_managed::ManagementData;
use enoki_exchange_shared::is_owned;
#[allow(unused_imports)]
use enoki_exchange_shared::is_owned::OwnershipData;

mod main_pool;
mod liquidity;
mod heartbeat;
mod upgrade;
mod shared_candid_methods;

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
