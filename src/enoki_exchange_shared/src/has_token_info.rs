use std::borrow::Borrow;
use std::cell::RefCell;

use candid::{candid_method, CandidType, Nat, Principal};
use ic_cdk_macros::*;

use crate::types::{EnokiToken, Result, TxError};

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug)]
pub struct TokenInfo {
    pub token_a: Principal,
    pub token_b: Principal,
}

impl Default for TokenInfo {
    fn default() -> Self {
        Self {
            token_a: Principal::anonymous(),
            token_b: Principal::anonymous(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug)]
pub struct AssignedShards {
    pub token_a: Principal,
    pub token_b: Principal,
}

impl Default for AssignedShards {
    fn default() -> Self {
        Self {
            token_a: Principal::anonymous(),
            token_b: Principal::anonymous(),
        }
    }
}

#[derive(Default)]
struct State {
    token_info: TokenInfo,
    assigned_shards: AssignedShards,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

pub fn export_stable_storage() -> (TokenInfo, AssignedShards) {
    let State {
        token_info,
        assigned_shards,
    } = STATE.with(|s| s.take());
    (token_info, assigned_shards)
}

pub fn import_stable_storage(token_info: TokenInfo, assigned_shards: AssignedShards) {
    STATE.with(|s| {
        s.replace(State {
            token_info,
            assigned_shards,
        })
    });
}

pub fn init_token_info(token_info: TokenInfo, assigned_shards: AssignedShards) {
    STATE.with(|s| {
        s.replace(State {
            token_info,
            assigned_shards,
        })
    });
}

pub async fn register(with_token: Principal) -> Result<Principal> {
    let result: Result<(Result<Principal>,)> =
        ic_cdk::call(with_token, "register", (ic_cdk::id(),))
            .await
            .map_err(|e| e.into());
    result?.0
}

#[query(name = "getTokenInfo")]
#[candid_method(query, rename = "getTokenInfo")]
pub fn get_token_info() -> TokenInfo {
    STATE.with(|s| s.borrow().token_info.clone())
}

pub fn get_assigned_shards() -> AssignedShards {
    STATE.with(|s| s.borrow().assigned_shards.clone())
}

pub fn get_assigned_shard(for_token: &EnokiToken) -> Principal {
    STATE.with(|s| match for_token {
        EnokiToken::TokenA => s.borrow().assigned_shards.token_a,
        EnokiToken::TokenB => s.borrow().assigned_shards.token_a,
    })
}

pub fn parse_from() -> Result<EnokiToken> {
    let caller = ic_cdk::caller();
    STATE.with(|s| {
        let s = s.borrow();
        if s.assigned_shards.token_a == caller {
            Ok(EnokiToken::TokenA)
        } else if s.assigned_shards.token_b == caller {
            Ok(EnokiToken::TokenB)
        } else {
            Err(TxError::Unauthorized)
        }
    })
}
