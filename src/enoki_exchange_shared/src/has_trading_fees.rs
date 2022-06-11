use std::cell::RefCell;

use candid::{CandidType, Nat};

use crate::types::{EnokiToken, StableNat};

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct TradingFees {
    pub token_a_deposit_fee: StableNat,
    // constant fee charged when swapping or submitting a limit order
    pub token_b_deposit_fee: StableNat,
    // constant fee charged when swapping or submitting a limit order
    pub limit_order_taker_fee: f64,
    // as percentage of trade quantity (ex: 0.002)
    pub swap_fee: f64,
    // as percentage of trade quantity (ex: 0.002)
    pub swap_market_maker_reward: f64,  // as percentage of the swap fee (ex: 0.3)
}

thread_local! {
    static STATE: RefCell<TradingFees> = RefCell::new(TradingFees::default());
}

pub fn export_stable_storage() -> TradingFees {
    STATE.with(|s| s.take())
}

pub fn import_stable_storage(data: TradingFees) {
    STATE.with(|s| s.replace(data));
}

pub fn init_fee_info(data: TradingFees) {
    if data.limit_order_taker_fee > 0.03 || data.limit_order_taker_fee < 0.0 {
        panic!("limit order taker fee out of range")
    }
    if data.swap_fee > 0.03 || data.swap_fee < 0.0 {
        panic!("swap fee out of range")
    }
    if data.swap_market_maker_reward > 1.00 || data.swap_market_maker_reward < 0.0 {
        panic!("swap market marker reward out of range")
    }


    STATE.with(|s| s.replace(data));
}

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
