use std::cell::RefCell;

use candid::{candid_method, CandidType, Nat, Principal};
use ic_cdk_macros::*;

use crate::types::{EnokiToken, StableNat};

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct TradingFees {
    pub token_a_deposit_fee: StableNat,
    pub token_b_deposit_fee: StableNat,
    pub limit_order_taker_fee: f64,
    pub swap_fee: f64,
    pub swap_market_maker_reward: f64,
}

thread_local! {
    static STATE: RefCell<TradingFees> = RefCell::new(TradingFees::default());
}

pub fn export_stable_storage() -> (TradingFees,) {
    let data = STATE.with(|s| s.take());
    (data,)
}

pub fn import_stable_storage(data: TradingFees) {
    STATE.with(|s| s.replace(data));
}

pub fn init_fee_info(data: TradingFees) {
    STATE.with(|s| s.replace(data));
}

#[query(name = "getTradingFees")]
#[candid_method(query, rename = "getTradingFees")]
pub fn get_trading_fees() -> TradingFees {
    STATE.with(|s| s.borrow().clone())
}

pub fn get_deposit_fee(token: &EnokiToken) -> Nat {
    STATE.with(|s| match token {
        EnokiToken::TokenA => s.borrow().token_a_deposit_fee.0.clone(),
        EnokiToken::TokenB => s.borrow().token_b_deposit_fee.0.clone(),
    })
}

pub fn get_limit_order_taker_fee() -> f64 {
    STATE.with(|s| s.borrow().limit_order_taker_fee)
}

pub fn get_swap_fee() -> f64 {
    STATE.with(|s| s.borrow().swap_fee)
}

pub fn get_swap_market_maker_reward() -> f64 {
    STATE.with(|s| s.borrow().swap_market_maker_reward)
}
