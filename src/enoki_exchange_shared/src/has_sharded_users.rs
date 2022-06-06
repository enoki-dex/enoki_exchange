use std::cell::RefCell;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Deserialize, Principal};
use ic_cdk_macros::*;

use crate::types::*;

pub fn register_user(user: ShardedPrincipal) {
    STATE.with(|s| s.borrow_mut().users.insert(user.principal, user.shard));
}

pub fn get_user_shard(user: Principal) -> Result<Principal> {
    STATE
        .with(|s| s.borrow().users.get(&user).copied())
        .ok_or(TxError::UserNotRegistered)
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct ShardedUserState {
    users: HashMap<Principal, Principal>,
}

thread_local! {
    static STATE: RefCell<ShardedUserState> = RefCell::new(Default::default());
}

pub fn export_stable_storage() -> (ShardedUserState,) {
    let data: ShardedUserState = STATE.with(|b| b.take());
    (data,)
}

pub fn import_stable_storage(data: ShardedUserState) {
    STATE.with(|b| b.replace(data));
}
