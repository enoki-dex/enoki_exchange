use std::cell::RefCell;

use candid::{candid_method, CandidType, Deserialize, Principal};
use ic_cdk_macros::*;

use crate::types::*;

#[derive(Deserialize, CandidType, Clone, Debug)]
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
        Err(TxError::Unauthorized)
    }
}

thread_local! {
    static STATE: RefCell<OwnershipData> = RefCell::new(OwnershipData::default());
}

#[query(name = "getOwner")]
#[candid_method(query, rename = "getOwner")]
fn get_owner() -> Principal {
    STATE.with(|d| d.borrow().owner)
}

#[update(name = "setOwner")]
#[candid_method(update, rename = "setOwner")]
fn set_owner(new_owner: Principal) -> Result<()> {
    STATE.with(|d| {
        let owner = &mut d.borrow_mut().owner;
        if ic_cdk::caller() == *owner {
            *owner = new_owner;
            Ok(())
        } else {
            Err(TxError::Unauthorized)
        }
    })
}

pub fn export_stable_storage() -> (OwnershipData,) {
    let data: OwnershipData = STATE.with(|b| b.take());
    (data,)
}

pub fn import_stable_storage(data: OwnershipData) {
    STATE.with(|b| b.replace(data));
}
