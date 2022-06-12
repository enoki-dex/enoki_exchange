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
fn init() {
    is_owned::init_owner(OwnershipData {
        owner: ic_cdk::caller(),
        deploy_time: ic_cdk::api::time(),
    });
}

#[update(name = "finishInit")]
#[candid_method(update, rename = "finishInit")]
fn finish_init(main_pool: Principal) {
    is_owned::assert_is_owner().unwrap();
    assert_eq!(is_managed::get_manager(), Principal::anonymous(), "already init");
    is_managed::init_manager(ManagementData { manager: main_pool });
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}
