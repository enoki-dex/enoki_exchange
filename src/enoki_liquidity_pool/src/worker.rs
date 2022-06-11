use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Deserialize, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::is_owned;
use enoki_exchange_shared::types::*;
use enoki_exchange_shared::{has_token_info, is_managed};
use enoki_exchange_shared::has_token_info::AssignedShards;

pub fn assert_is_worker_contract() -> Result<()> {
    if STATE.with(|s| s.borrow().worker_id == ic_cdk::caller()) {
        Ok(())
    } else {
        Err(TxError::Unauthorized)
    }
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug)]
pub struct WorkerContractData {
    pub worker_id: Principal,
    pub worker_shard: Principal,
}

impl Default for WorkerContractData {
    fn default() -> Self {
        Self {
            worker_id: Principal::anonymous(),
            worker_shard: Principal::anonymous(),
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
async fn init_worker(worker: Principal) -> Result<()> {
    is_owned::assert_is_owner()?;
    let response: Result<(Principal,)> =
        ic_cdk::call(worker, "initWorker", (has_token_info::get_token_info(),))
            .await
            .map_err(|e| e.into());
    let worker_shard = response?.0;
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.worker_id = worker;
        s.worker_shard = worker_shard;
    });
    Ok(())
}

#[update(name = "addBroker")]
#[candid_method(update, rename = "addBroker")]
async fn add_broker(broker: Principal) -> Result<()> {
    is_managed::assert_is_manager()?;

    let result: Result<()> = ic_cdk::call(get_worker(), "addBroker", (broker,))
        .await
        .map_err(|e| e.into());
    result
}

pub fn get_worker_shard() -> Principal {
    STATE.with(|s| s.borrow().worker_shard)
}

pub fn export_stable_storage() -> WorkerContractData {
    STATE.with(|b| b.take())
}

pub fn import_stable_storage(data: WorkerContractData) {
    STATE.with(|b| b.replace(data));
}
