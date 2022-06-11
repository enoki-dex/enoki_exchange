use std::collections::{BTreeMap, HashMap};
use std::ops::{Sub, SubAssign};

use candid::{candid_method, CandidType, Nat, Principal};

use enoki_exchange_shared::has_token_info::QuantityTranslator;
use enoki_exchange_shared::has_trading_fees::get_limit_order_taker_fee;
use enoki_exchange_shared::types::*;
use enoki_exchange_shared::utils::{nat_div_float, nat_x_float};

pub trait OrderMatching {
    fn try_execute(&mut self, order_quantity_token: &EnokiToken, executor: &mut Self);
    fn try_buy_from(&mut self, executor: &mut Self) {
        self.try_execute(&EnokiToken::TokenB, executor)
    }
    fn try_sell_to(&mut self, executor: &mut Self) {
        self.try_execute(&EnokiToken::TokenA, executor)
    }
    fn is_complete(&self) -> bool;
}

impl OrderMatching for Order {
    fn try_execute(&mut self, order_quantity_token: &EnokiToken, executor: &mut Self) {
        if let OrderStatus::Pending = self.state.status {
            if let OrderStatus::Pending = executor.state.status {
                let taker_fee = get_limit_order_taker_fee();
                let mut quantity_remaining = std::mem::take(&mut self.state.quantity_remaining.0);
                quantity_remaining = nat_x_float(quantity_remaining, 1.0 - taker_fee).unwrap();
                let market_maker_original_quantity = executor.state.quantity_remaining.clone();
                let (mut quantity_translator, quantity_b) = match order_quantity_token {
                    EnokiToken::TokenA => (
                        QuantityTranslator::new(executor.info.limit_price, &mut quantity_remaining),
                        &mut executor.state.quantity_remaining.0,
                    ),
                    EnokiToken::TokenB => (
                        QuantityTranslator::new(
                            executor.info.limit_price,
                            &mut executor.state.quantity_remaining.0,
                        ),
                        &mut quantity_remaining,
                    ),
                };
                let quantity_b_traded = quantity_translator
                    .get_quantity_b()
                    .unwrap()
                    .min(quantity_b.clone());
                *quantity_b -= quantity_b_traded.clone();
                quantity_translator.sub_assign(quantity_b_traded).unwrap();
                if quantity_remaining == 0u32 {
                    self.state.status = OrderStatus::Completed;
                } else {
                    self.state.quantity_remaining =
                        nat_div_float(quantity_remaining, 1.0 - taker_fee)
                            .unwrap()
                            .into();
                }
                if !executor.state.quantity_remaining.is_nonzero() {
                    executor.state.status = OrderStatus::Completed;
                }
                self.state.marker_makers.push(CounterpartyInfo {
                    broker: executor.info.broker,
                    user: executor.info.user,
                    quantity: market_maker_original_quantity
                        .sub(executor.state.quantity_remaining.clone()),
                    price: executor.info.limit_price,
                });
            }
        }
    }
    fn is_complete(&self) -> bool {
        if let OrderStatus::Pending = self.state.status {
            false
        } else {
            true
        }
    }
}
