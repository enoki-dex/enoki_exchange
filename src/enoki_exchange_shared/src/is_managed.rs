use std::cell::RefCell;

use candid::{candid_method, CandidType, Principal};
use ic_cdk_macros::*;

use crate::types::*;

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug)]
pub struct ManagementData {
    pub manager: Principal,
}

impl Default for ManagementData {
    fn default() -> Self {
        Self {
            manager: Principal::anonymous(),
        }
    }
}

pub fn init_manager(data: ManagementData) {
    STATE.with(|d| {
        *d.borrow_mut() = data;
    });
}

pub fn get_manager() -> Principal {
    STATE.with(|d| d.borrow().manager)
}

pub fn assert_is_manager() -> Result<()> {
    if STATE.with(|s| s.borrow().manager) == ic_cdk::caller() {
        Ok(())
    } else {
        Err(TxError::Unauthorized)
    }
}

thread_local! {
    static STATE: RefCell<ManagementData> = RefCell::new(ManagementData::default());
}

#[query(name = "getManagerContract")]
#[candid_method(query, rename = "getManagerContract")]
fn get_owner() -> Principal {
    STATE.with(|d| d.borrow().manager)
}

#[update(name = "setManagerContract")]
#[candid_method(update, rename = "setManagerContract")]
fn set_owner(new_owner: Principal) -> Result<()> {
    STATE.with(|d| {
        let owner = &mut d.borrow_mut().manager;
        if ic_cdk::caller() == *owner {
            *owner = new_owner;
            Ok(())
        } else {
            Err(TxError::Unauthorized)
        }
    })
}

pub fn export_stable_storage() -> ManagementData {
    STATE.with(|b| b.take())
}

pub fn import_stable_storage(data: ManagementData) {
    STATE.with(|b| b.replace(data));
}
