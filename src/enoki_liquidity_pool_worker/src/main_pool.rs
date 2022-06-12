use candid::{candid_method, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::has_token_info::AssignedShards;
use enoki_exchange_shared::is_managed;

#[update(name = "initWorker")]
#[candid_method(update, rename = "initWorker")]
async fn init_worker(supply_token_info: has_token_info::TokenPairInfo) -> AssignedShards {
    is_managed::assert_is_manager().unwrap();
    has_token_info::start_init_token_info(supply_token_info);
    has_token_info::finish_init_token_info().await.unwrap();
    has_token_info::get_assigned_shards()
}

#[update(name = "addBroker")]
#[candid_method(update, rename = "addBroker")]
async fn add_broker(broker: Principal) {
    is_managed::assert_is_manager().unwrap();

    has_token_info::add_token_spender(broker).await.unwrap();
}