use std::borrow::Borrow;
use std::cell::RefCell;

use candid::{candid_method, CandidType, Nat, Principal};
use ic_cdk_macros::*;

use crate::types::{EnokiToken, Result, StableNat, TxError};

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct TokenPairInfo {
    pub token_a: TokenInfo,
    pub token_b: TokenInfo,
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug)]
pub struct TokenInfo {
    pub principal: Principal,
    pub units_per_lot: StableNat,
    pub min_price_interval_lots: u64,
}

impl Default for TokenInfo {
    fn default() -> Self {
        Self {
            principal: Principal::anonymous(),
            units_per_lot: Default::default(),
            min_price_interval_lots: 0
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
    token_info: TokenPairInfo,
    assigned_shards: AssignedShards,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

pub fn export_stable_storage() -> (TokenPairInfo, AssignedShards) {
    let State {
        token_info,
        assigned_shards,
    } = STATE.with(|s| s.take());
    (token_info, assigned_shards)
}

pub fn import_stable_storage(token_info: TokenPairInfo, assigned_shards: AssignedShards) {
    STATE.with(|s| {
        s.replace(State {
            token_info,
            assigned_shards,
        })
    });
}

pub async fn init_token_info(token_info: TokenPairInfo) -> Result<()> {
    let (token_a, token_b) = register_tokens(&token_info).await?;
    STATE.with(|s| {
        s.replace(State {
            token_info,
            assigned_shards: AssignedShards { token_a, token_b },
        })
    });
    Ok(())
}

async fn register_tokens(token_info: &TokenPairInfo) -> Result<(Principal, Principal)> {
    let (assigned_a, assigned_b) =
        futures::future::join(register(token_info.token_a.principal), register(token_info.token_b.principal)).await;
    Ok((assigned_a?, assigned_b?))
}

async fn register(with_token: Principal) -> Result<Principal> {
    let result: Result<(Result<Principal>,)> =
        ic_cdk::call(with_token, "register", (ic_cdk::id(),))
            .await
            .map_err(|e| e.into());
    result?.0
}

#[query(name = "getTokenInfo")]
#[candid_method(query, rename = "getTokenInfo")]
pub fn get_token_info() -> TokenPairInfo {
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
