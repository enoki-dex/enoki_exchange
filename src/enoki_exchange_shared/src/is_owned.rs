use std::cell::RefCell;

use candid::{CandidType, Principal};

use crate::types::*;

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug)]
pub struct OwnershipData {
    pub owner: Principal,
    pub deploy_time: u64,
}

impl Default for OwnershipData {
    fn default() -> Self {
        Self {
            owner: Principal::anonymous(),
            deploy_time: 0,
        }
    }
}

pub fn init_owner(data: OwnershipData) {
    STATE.with(|d| {
        *d.borrow_mut() = data;
    });
}

pub fn assert_is_owner() -> Result<()> {
    if STATE.with(|s| s.borrow().owner) == ic_cdk::caller() {
        Ok(())
    } else {
        Err(TxError::Unauthorized.into())
    }
}

thread_local! {
    static STATE: RefCell<OwnershipData> = RefCell::new(OwnershipData::default());
}

pub fn get_owner() -> Principal {
    STATE.with(|d| d.borrow().owner)
}

pub fn set_owner(new_owner: Principal) -> Result<()> {
    STATE.with(|d| {
        let owner = &mut d.borrow_mut().owner;
        if ic_cdk::caller() == *owner {
            *owner = new_owner;
            Ok(())
        } else {
            Err(TxError::Unauthorized.into())
        }
    })
}

pub fn export_stable_storage() -> OwnershipData {
    STATE.with(|b| b.take())
}

pub fn import_stable_storage(data: OwnershipData) {
    STATE.with(|b| b.replace(data));
}
