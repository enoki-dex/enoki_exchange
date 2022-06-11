use std::borrow::Borrow;
use std::cell::RefCell;
use std::ops::{Div, Mul, Rem};

use candid::{candid_method, CandidType, Nat, Principal};
use ic_cdk_macros::*;
use num_traits::cast::ToPrimitive;
use num_traits::Pow;

use crate::types::{EnokiToken, Result, StableNat, TxError};
use crate::utils::{nat_div_float, nat_x_float};

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct TokenPairInfo {
    pub token_a: TokenInfo,
    pub token_b: TokenInfo,
    pub price_number_of_decimals: u64,
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug)]
pub struct TokenInfo {
    pub principal: Principal,
}

impl Default for TokenInfo {
    fn default() -> Self {
        Self {
            principal: Principal::anonymous(),
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

#[derive(serde::Serialize, serde::Deserialize, CandidType, Default)]
pub struct TokenInfoState {
    token_info: TokenPairInfo,
    assigned_shards: AssignedShards,
}

thread_local! {
    static STATE: RefCell<TokenInfoState> = RefCell::new(TokenInfoState::default());
}

pub fn export_stable_storage() -> TokenInfoState {
    let data = STATE.with(|s| s.take());
    data
}

pub fn import_stable_storage(data: TokenInfoState) {
    STATE.with(|s| s.replace(data));
}

pub async fn init_token_info(token_info: TokenPairInfo) -> Result<()> {
    let (token_a, token_b) = register_tokens(&token_info).await?;
    STATE.with(|s| {
        s.replace(TokenInfoState {
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

pub async fn add_token_spender(principal: Principal) -> Result<()> {
    let shards = get_assigned_shards();
    let result: Result<()> = ic_cdk::call(shards.token_a, "addSpender", (principal,))
        .await
        .map_err(|e| e.into());
    result?;
    let result: Result<()> = ic_cdk::call(shards.token_b, "addSpender", (principal,))
        .await
        .map_err(|e| e.into());
    result
}

pub fn get_token_address(token: &EnokiToken) -> Principal {
    STATE.with(|s| s.borrow().token_info.get(token).principal)
}

#[query(name = "getAssignedShards")]
#[candid_method(query, rename = "getAssignedShards")]
pub fn get_assigned_shards() -> AssignedShards {
    STATE.with(|s| s.borrow().assigned_shards.clone())
}

pub fn get_assigned_shard(for_token: &EnokiToken) -> Principal {
    STATE.with(|s| match for_token {
        EnokiToken::TokenA => s.borrow().assigned_shards.token_a,
        EnokiToken::TokenB => s.borrow().assigned_shards.token_a,
    })
}

pub fn price_in_b_float_to_u64(value: f64) -> Result<u64> {
    STATE.with(|s| {
        let s = s.borrow();
        let num_decimals = s.token_info.price_number_of_decimals;
        let value_int = (value * 10f64.pow(num_decimals as f64)) as u64;
        // if (value_int as f64) / 10f64.pow(num_decimals as f64) != value {
        //     return Err(TxError::IntUnderflow);
        // }
        Ok(value_int)
    })
}

pub fn price_in_b_u64_to_float(value: u64) -> f64 {
    STATE.with(|s| {
        let num_decimals = s.borrow().token_info.price_number_of_decimals;
        (value as f64) / 10f64.pow(num_decimals as f64)
    })
}

pub fn quant_b_to_quant_a(quant_b: Nat, price: u64) -> Result<Nat> {
    let price = price_in_b_u64_to_float(price);
    nat_div_float(quant_b, price)
}

pub fn quant_a_to_quant_b(quant_a: Nat, price: u64) -> Result<Nat> {
    let price = price_in_b_u64_to_float(price);
    nat_x_float(quant_a, price)
}

pub struct QuantityTranslator<'a> {
    quantity_a: &'a mut Nat,
    price: u64,
}

impl<'a> QuantityTranslator<'a> {
    pub fn new(price: u64, quantity_a: &'a mut Nat) -> Self {
        Self { price, quantity_a }
    }
    pub fn get_quantity_b(&self) -> Result<Nat> {
        quant_a_to_quant_b(self.quantity_a.clone(), self.price)
    }
    pub fn sub_assign(&mut self, quantity_b: Nat) -> Result<()> {
        let current = self.get_quantity_b()?;
        *self.quantity_a = quant_b_to_quant_a(current - quantity_b, self.price)?;
        Ok(())
    }
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
