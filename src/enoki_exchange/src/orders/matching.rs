use std::collections::{BTreeMap, HashMap};
use std::ops::{Sub, SubAssign};

use candid::{candid_method, CandidType, Nat, Principal};

use enoki_exchange_shared::has_token_info::QuantityTranslator;
use enoki_exchange_shared::types::*;

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
                let market_maker_original_quantity = executor.state.quantity_remaining.clone();
                let (mut quantity_translator, quantity_b) = match order_quantity_token {
                    EnokiToken::TokenA => (
                        QuantityTranslator::new(
                            executor.info.limit_price,
                            &mut self.state.quantity_remaining.0,
                        ),
                        &mut executor.state.quantity_remaining,
                    ),
                    EnokiToken::TokenB => (
                        QuantityTranslator::new(
                            executor.info.limit_price,
                            &mut executor.state.quantity_remaining.0,
                        ),
                        &mut self.state.quantity_remaining,
                    ),
                };
                let quantity_b_traded = quantity_translator
                    .get_quantity_b()
                    .unwrap()
                    .min(quantity_b.0.clone());
                quantity_b.0 -= quantity_b_traded.clone();
                quantity_translator.sub_assign(quantity_b_traded).unwrap();
                if !self.state.quantity_remaining.is_nonzero() {
                    self.state.status = OrderStatus::Completed;
                }
                if !executor.state.quantity_remaining.is_nonzero() {
                    executor.state.status = OrderStatus::Completed;
                }
                self.state.marker_makers.push(CounterpartyInfo {
                    broker: executor.info.broker,
                    user: executor.info.user,
                    quantity: market_maker_original_quantity
                        .sub(executor.state.quantity_remaining.clone()),
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
