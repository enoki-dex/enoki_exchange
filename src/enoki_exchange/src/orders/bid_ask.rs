use std::collections::{BTreeMap, HashMap};

use candid::{candid_method, CandidType, Nat, Principal};

use enoki_exchange_shared::types::*;

use crate::orders::matching::OrderMatching;

#[derive(serde::Deserialize, serde::Serialize, CandidType, Clone, Debug, Default)]
pub struct BidAsk(BTreeMap<u64, Vec<Order>>);

impl BidAsk {
    pub fn get_counterparty_info(&self) -> BTreeMap<u64, Vec<CounterpartyInfo>> {
        self.0
            .iter()
            .map(|(&price, orders)| {
                (
                    price,
                    orders
                        .into_iter()
                        .map(|order| CounterpartyInfo {
                            broker: order.info.broker,
                            user: order.info.user,
                            quantity: order.state.quantity_remaining.clone(),
                        })
                        .collect(),
                )
            })
            .collect()
    }
    pub fn get_highest_price(&self) -> Option<u64> {
        self.0.keys().last().copied()
    }
    pub fn get_lowest_price(&self) -> Option<u64> {
        self.0.keys().next().copied()
    }
    pub fn insert(&mut self, order: Order) {
        self.0
            .entry(order.info.limit_price)
            .or_default()
            .push(order);
    }
    pub fn try_cancel(&mut self, order: &OrderInfo) -> Option<Order> {
        if let Some(orders) = self.0.get_mut(&order.limit_price) {
            if let Some(index) = orders
                .iter()
                .position(|o| o.info.broker == order.broker && o.info.id == order.id)
            {
                let mut cancelled = orders.remove(index);
                if let OrderStatus::Pending = cancelled.state.status {
                    cancelled.state.status = OrderStatus::Cancelled;
                }
                if orders.is_empty() {
                    self.0.remove(&order.limit_price);
                }
                return Some(cancelled);
            }
        }
        None
    }
    pub fn try_match_with_asks(&mut self, order: &mut Order) {
        for (&price, market) in self.0.iter_mut() {
            if price > order.info.limit_price {
                break;
            }
            for executor in market.iter_mut() {
                order.try_buy_from(executor);
            }
        }
    }
    pub fn try_match_with_bids(&mut self, order: &mut Order) {
        for (&price, market) in self.0.iter_mut().rev() {
            if price < order.info.limit_price {
                break;
            }
            for executor in market.iter_mut() {
                order.try_sell_to(executor);
            }
        }
    }
    pub fn cancel_expired(&mut self) {
        let now = ic_cdk::api::time();
        for order in self
            .0
            .iter_mut()
            .flat_map(|(_, orders)| orders)
            .filter(|order| {
                if let Some(expiry) = order.info.expiration_time {
                    expiry <= now
                } else {
                    false
                }
            })
        {
            order.state.status = OrderStatus::Expired;
        }
    }
    pub fn take_completed(&mut self) -> Vec<Order> {
        let mut complete = Vec::new();
        let mut empty = Vec::new();
        for (&price, orders) in self.0.iter_mut() {
            for i in (0..orders.len()).rev() {
                if orders[i].is_complete() {
                    complete.push(orders.remove(i));
                }
            }
            if orders.is_empty() {
                empty.push(price);
            }
        }
        for price in empty {
            self.0.remove(&price);
        }

        complete
    }
}
