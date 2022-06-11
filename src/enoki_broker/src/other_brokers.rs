use std::cell::RefCell;
use std::collections::HashSet;

use candid::{candid_method, CandidType, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::is_managed::assert_is_manager;
use enoki_exchange_shared::types::*;

thread_local! {
    static STATE: RefCell<BrokersState> = RefCell::new(BrokersState::default());
}

pub fn assert_is_broker(principal: Principal) -> Result<()> {
    if STATE.with(|s| s.borrow().other_brokers.contains(&principal)) {
        Ok(())
    } else {
        Err(TxError::Unauthorized)
    }
}

#[update(name = "addBroker")]
#[candid_method(update, rename = "addBroker")]
fn add_broker(principal: Principal) {
    assert_is_manager().unwrap();
    STATE.with(|s| s.borrow_mut().other_brokers.insert(principal));
}

pub fn init_brokers(brokers: Vec<Principal>) {
    STATE.with(|s| s.borrow_mut().other_brokers.extend(brokers));
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct BrokersState {
    other_brokers: HashSet<Principal>,
}

pub fn export_stable_storage() -> BrokersState {
    STATE.with(|s| s.take())
}

pub fn import_stable_storage(data: BrokersState) {
    STATE.with(|s| s.replace(data));
}
