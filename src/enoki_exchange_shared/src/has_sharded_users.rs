use std::cell::RefCell;
use std::collections::HashMap;

use candid::{CandidType, Principal};

use crate::types::*;

pub fn register_user(user: Principal, token: Principal, assigned_shard: Principal) {
    STATE.with(|s| s.borrow_mut().users.insert(UserAndToken { user, token }, assigned_shard));
}

pub fn get_user_shard(user: Principal, token: Principal) -> Result<Principal> {
    STATE
        .with(|s| s.borrow().users.get(&UserAndToken { user, token }).copied())
        .ok_or(TxError::UserNotRegistered)
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct ShardedUserState {
    users: HashMap<UserAndToken, Principal>,
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Hash, Eq, PartialEq)]
struct UserAndToken {
    user: Principal,
    token: Principal,
}

thread_local! {
    static STATE: RefCell<ShardedUserState> = RefCell::new(Default::default());
}

pub fn export_stable_storage() -> ShardedUserState {
    STATE.with(|b| b.take())
}

pub fn import_stable_storage(data: ShardedUserState) {
    STATE.with(|b| b.replace(data));
}
