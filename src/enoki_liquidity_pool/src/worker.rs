use std::cell::RefCell;

use candid::{candid_method, CandidType, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info::AssignedShards;
use enoki_exchange_shared::types::*;
use enoki_exchange_shared::{has_sharded_users, is_owned};
use enoki_exchange_shared::{has_token_info, is_managed};

pub fn assert_is_worker_contract() -> Result<()> {
    if STATE.with(|s| s.borrow().worker_id == ic_cdk::caller()) {
        Ok(())
    } else {
        Err(TxError::Unauthorized.into())
    }
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug)]
pub struct WorkerContractData {
    pub worker_id: Principal,
}

impl Default for WorkerContractData {
    fn default() -> Self {
        Self {
            worker_id: Principal::anonymous(),
        }
    }
}

thread_local! {
    static STATE: RefCell<WorkerContractData> = RefCell::new(WorkerContractData::default());
}

#[query(name = "getWorker")]
#[candid_method(query, rename = "getWorker")]
pub fn get_worker() -> Principal {
    STATE.with(|d| d.borrow().worker_id)
}

#[update(name = "initWorker")]
#[candid_method(update, rename = "initWorker")]
async fn init_worker(worker: Principal) {
    is_owned::assert_is_owner().unwrap();
    if STATE.with(|s| s.borrow().worker_id != Principal::anonymous()) {
        panic!("worker already init");
    }
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.worker_id = worker;
    });
}

pub async fn init_worker_token_data() -> Result<()> {
    let worker = STATE.with(|s| s.borrow().worker_id);
    let response: Result<(AssignedShards,)> =
        ic_cdk::call(worker, "initWorker", (has_token_info::get_token_info(),))
            .await
            .map_err(|e| e.into_tx_error());
    let worker_shards = response?.0;
    has_sharded_users::register_user_with(
        worker,
        has_token_info::get_token_address(&EnokiToken::TokenA),
        worker_shards.token_a,
    );
    has_sharded_users::register_user_with(
        worker,
        has_token_info::get_token_address(&EnokiToken::TokenB),
        worker_shards.token_b,
    );
    Ok(())
}

#[update(name = "addBroker")]
#[candid_method(update, rename = "addBroker")]
async fn add_broker(broker: Principal) {
    is_managed::assert_is_manager().unwrap();

    let result: Result<()> = ic_cdk::call(get_worker(), "addBroker", (broker,))
        .await
        .map_err(|e| e.into_tx_error());
    result.unwrap();
}

pub fn _get_worker_shard(token: &EnokiToken) -> Result<Principal> {
    let worker = STATE.with(|s| s.borrow().worker_id);
    has_sharded_users::get_user_shard(worker, has_token_info::get_token_address(token))
}

pub fn export_stable_storage() -> WorkerContractData {
    STATE.with(|b| b.take())
}

pub fn import_stable_storage(data: WorkerContractData) {
    STATE.with(|b| b.replace(data));
}
