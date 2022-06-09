use std::borrow::Borrow;
use std::cell::RefCell;
use std::ops::{Div, Mul, Rem};

use candid::{candid_method, CandidType, Nat, Principal};
use ic_cdk_macros::*;
use num_traits::cast::ToPrimitive;

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
            min_price_interval_lots: 0,
        }
    }
}

impl TokenPairInfo {
    pub fn get(&self, token: &EnokiToken) -> &TokenInfo {
        match token {
            EnokiToken::TokenA => &self.token_a,
            EnokiToken::TokenB => &self.token_b,
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
    let (assigned_a, assigned_b) = futures::future::join(
        register(token_info.token_a.principal),
        register(token_info.token_b.principal),
    )
    .await;
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

pub fn nat_to_lots(token: &EnokiToken, value: Nat, is_price: bool) -> Result<u64> {
    STATE.with(|s| {
        let s = s.borrow();
        let info = s.token_info.get(token);
        if value.clone().rem(info.units_per_lot.0.clone()) != 0 {
            return Err(TxError::IntUnderflow);
        }
        let value_lots = value.div(info.units_per_lot.0.clone());
        let value64 = match value_lots.0.to_u64().ok_or(TxError::IntOverflow) {
            Ok(val) => val,
            Err(err) => return Err(err),
        };
        if is_price && value64 % info.min_price_interval_lots != 0 {
            Err(TxError::IntUnderflow)
        } else {
            Ok(value64)
        }
    })
}

pub fn lots_to_nat(token: &EnokiToken, value: u64) -> Nat {
    STATE.with(|s| {
        s.borrow()
            .token_info
            .get(token)
            .units_per_lot
            .0
            .clone()
            .mul(value)
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
