use std::cell::RefCell;
use std::collections::HashSet;

use candid::{candid_method, CandidType, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::is_managed::assert_is_manager;
use enoki_exchange_shared::types::*;

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct UsersState {
    users: HashSet<Principal>,
}

thread_local! {
    static STATE: RefCell<UsersState> = RefCell::new(UsersState::default());
}

pub fn assert_is_user(principal: Principal) -> Result<()> {
    if STATE.with(|s| s.borrow().users.contains(&principal)) {
        Ok(())
    } else {
        Err(TxError::UserNotRegistered {
            user: principal.to_string(),
            registry: ic_cdk::id().to_string(),
        }
        .into())
    }
}

#[update(name = "addUser")]
#[candid_method(update, rename = "addUser")]
fn add_user(principal: Principal) {
    assert_is_manager().unwrap();
    STATE.with(|s| s.borrow_mut().users.insert(principal));
}

pub fn export_stable_storage() -> UsersState {
    STATE.with(|s| s.take())
}

pub fn import_stable_storage(data: UsersState) {
    STATE.with(|s| s.replace(data));
}
