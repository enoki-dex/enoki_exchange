use std::cell::RefCell;

use candid::{CandidType, Principal};

use crate::is_owned::assert_is_owner;
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

pub fn assert_is_manager() -> Result<()> {
    if STATE.with(|s| s.borrow().manager) == ic_cdk::caller() {
        Ok(())
    } else {
        Err(TxError::Unauthorized.into())
    }
}

thread_local! {
    static STATE: RefCell<ManagementData> = RefCell::new(ManagementData::default());
}

pub fn get_manager() -> Principal {
    STATE.with(|d| d.borrow().manager)
}

pub fn set_manager(new_manager: Principal) -> Result<()> {
    assert_is_owner()?;
    STATE.with(|d| d.borrow_mut().manager = new_manager);
    Ok(())
}

pub fn export_stable_storage() -> ManagementData {
    STATE.with(|b| b.take())
}

pub fn import_stable_storage(data: ManagementData) {
    STATE.with(|b| b.replace(data));
}
