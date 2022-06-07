use std::cell::RefCell;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Deserialize, Principal};
use ic_cdk_macros::*;
use enoki_exchange_shared::has_sharded_users::{get_user_shard, register_user};

use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::is_owned;
use enoki_exchange_shared::types::*;

pub fn assert_is_worker_contract() -> Result<()> {
    if WORKERS.with(|s| s.borrow().workers.contains_key(&ic_cdk::caller())) {
        Ok(())
    } else {
        Err(TxError::Unauthorized)
    }
}

#[derive(Deserialize, CandidType, Clone, Debug)]
pub struct Worker {
    pub id: Principal,
}

#[derive(Deserialize, CandidType, Clone, Debug, Default)]
pub struct WorkerContractData {
    pub workers: HashMap<Principal, Worker>,
}

thread_local! {
    static WORKERS: RefCell<WorkerContractData> = RefCell::new(WorkerContractData::default());
}

#[query(name = "getWorkers")]
#[candid_method(query, rename = "getWorkers")]
fn get_workers() -> WorkerContractData {
    WORKERS.with(|d| d.borrow().clone())
}

#[update(name = "addWorker")]
#[candid_method(update, rename = "addWorker")]
async fn add_worker(worker: Principal) -> Result<()> {
    is_owned::assert_is_owner()?;
    let response: Result<(Principal,)> =
        ic_cdk::call(worker, "initWorker", (has_token_info::get_token_info(),))
            .await
            .map_err(|e| e.into());
    let worker_shard = response?.0;
    WORKERS.with(|s| s.borrow_mut().workers.insert(worker, Worker { id: worker }));
    register_user(ShardedPrincipal {
        shard: worker_shard,
        principal: worker
    });
    Ok(())
}

pub fn get_worker_shard(worker: Principal) -> Result<Principal> {
    get_user_shard(worker)
}

pub fn export_stable_storage() -> (WorkerContractData,) {
    let data: WorkerContractData = WORKERS.with(|b| b.take());
    (data,)
}

pub fn import_stable_storage(data: WorkerContractData) {
    WORKERS.with(|b| b.replace(data));
}
