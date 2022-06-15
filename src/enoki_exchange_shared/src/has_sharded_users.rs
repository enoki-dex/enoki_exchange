use std::cell::RefCell;
use std::collections::HashMap;

use candid::{CandidType, Principal};

use crate::has_token_info;
use crate::types::*;

pub fn register_user_with(user: Principal, token: Principal, assigned_shard: Principal) {
    STATE.with(|s| {
        s.borrow_mut()
            .users
            .insert(UserAndToken { user, token }, assigned_shard)
    });
}

pub async fn register_user(user: Principal) -> Result<()> {
    let (resp1, resp2) = futures::future::join(
        register_user_for_token(user, &EnokiToken::TokenA),
        register_user_for_token(user, &EnokiToken::TokenB),
    )
    .await;
    resp1.unwrap();
    resp2.unwrap();

    Ok(())
}

async fn register_user_for_token(user: Principal, token: &EnokiToken) -> Result<()> {
    let token_principal = has_token_info::get_token_address(token);
    let response: Result<(Principal,)> = ic_cdk::call(token_principal, "register", (user,))
        .await
        .map_err(|e| e.into_tx_error().into());
    register_user_with(user, token_principal, response?.0);
    Ok(())
}

pub fn get_user_shard(user: Principal, token: Principal) -> Result<Principal> {
    STATE
        .with(|s| s.borrow().users.get(&UserAndToken { user, token }).copied())
        .ok_or(
            TxError::UserNotRegistered {
                user: user.to_string(),
                registry: ic_cdk::id().to_string(),
            }
            .into(),
        )
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
