use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashMap;

use crate::types::*;
use candid::{candid_method, CandidType, Nat, Principal};
use ic_cdk_macros::*;

pub mod liquidity_pool;
pub mod single_user_liquidity_pool;

#[derive(serde::Deserialize, serde::Serialize, CandidType, Clone, Debug, Default)]
pub struct RequestForLiquidityChanges {
    pub to_add: LiquidityAmount,
    pub to_remove: LiquidityAmount,
}

#[derive(serde::Deserialize, serde::Serialize, CandidType, Clone, Debug, Default)]
pub struct ResponseAboutLiquidityChanges {
    pub added: LiquidityAmount,
    pub removed: LiquidityAmount,
    pub traded: LiquidityTrades,
}

#[derive(serde::Deserialize, serde::Serialize, CandidType, Clone, Debug, Default)]
pub struct RequestForNewLiquidityTarget {
    pub target: LiquidityAmount,
    pub extra_liquidity_available: LiquidityAmount,
}
