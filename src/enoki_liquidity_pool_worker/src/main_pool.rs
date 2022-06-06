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
async fn init_worker(supply_token_info: has_token_info::TokenInfo) -> Result<()> {
    is_managed::assert_is_manager()?;
    let (assigned_a, assigned_b) = tokio::join!(
        has_token_info::register(supply_token_info.token_a),
        has_token_info::register(supply_token_info.token_b)
    );
    has_token_info::init_token_info(
        supply_token_info,
        AssignedShards {
            token_a: assigned_a?,
            token_b: assigned_b?,
        },
    );
    Ok(())
}