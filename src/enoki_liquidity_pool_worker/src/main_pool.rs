use std::cell::RefCell;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Deserialize, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::has_token_info::AssignedShards;
use enoki_exchange_shared::is_managed;
use enoki_exchange_shared::types::*;

#[update(name = "initWorker")]
#[candid_method(update, rename = "initWorker")]
async fn init_worker(supply_token_info: has_token_info::TokenPairInfo) -> Result<()> {
    is_managed::assert_is_manager()?;
    has_token_info::init_token_info(supply_token_info).await?;
    Ok(())
}

#[update(name = "addBroker")]
#[candid_method(update, rename = "addBroker")]
async fn add_broker(broker: Principal) -> Result<()> {
    is_managed::assert_is_manager()?;

    has_token_info::add_token_spender(broker).await?;
    Ok(())
}
