use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashMap;

use crate::types::*;
use candid::{candid_method, CandidType, Nat, Principal};
use ic_cdk_macros::*;

pub mod liquidity_pool;

#[derive(serde::Deserialize, serde::Serialize, CandidType, Clone, Debug, Default)]
pub struct RequestForLiquidityChanges {
    pub to_add: HashMap<Principal, LiquidityAmount>,
    pub to_remove: HashMap<Principal, LiquidityAmount>,
}

#[derive(serde::Deserialize, serde::Serialize, CandidType, Clone, Debug, Default)]
pub struct ResponseAboutLiquidityChanges {
    pub added: HashMap<Principal, LiquidityAmount>,
    pub removed: HashMap<Principal, LiquidityAmount>,
    pub traded: HashMap<Principal, LiquidityTrades>,
}
