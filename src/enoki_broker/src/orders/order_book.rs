use std::collections::HashMap;

use candid::{CandidType, Principal};

use enoki_exchange_shared::types::*;

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct OrderBook {
    last_id: u64,
    orders: HashMap<u64, OrderInfo>,
    pending_orders: HashMap<u64, OrderInfo>,
    orders_to_cancel: HashMap<u64, OrderInfo>,
    pending_orders_to_cancel: HashMap<u64, OrderInfo>,
}

impl OrderBook {
    pub fn get_open_orders(&self, ids: &[u64]) -> OpenOrderStatus {
        OpenOrderStatus {
            open_orders: ids
                .iter()
                .filter_map(|id| self.pending_orders.get(id).or(self.orders.get(id)))
                .cloned()
                .map(|order| order.into())
                .collect(),
            pending_cancel: ids
                .iter()
                .filter(|&id| {
                    self.pending_orders_to_cancel.contains_key(id)
                        || self.orders_to_cancel.contains_key(id)
                })
                .copied()
                .collect(),
        }
    }
    pub fn create_limit_order(&mut self, input: ProcessedOrderInput) -> (Principal, u64) {
        self.create_order(input)
    }
    pub fn _create_market_order(&mut self, input: ProcessedOrderInput) -> (Principal, u64) {
        self.create_order(input)
    }
    fn get_next_id(&mut self) -> u64 {
        self.last_id += 1;
        self.last_id
    }
    fn create_order(&mut self, input: ProcessedOrderInput) -> (Principal, u64) {
        let id = self.get_next_id();
        let order = OrderInfo {
            broker: ic_cdk::id(),
            user: input.user,
            id,
            side: input.side,
            maker_taker: input.maker_taker,
            limit_price: input.limit_price_in_b,
            quantity: input.quantity.into(),
            expiration_time: input.expiration_time,
        };
        self.pending_orders.insert(id, order);
        (input.user, id)
    }
    pub fn lock_pending_orders(&mut self) -> (Vec<OrderInfo>, Vec<OrderInfo>) {
        let orders = self.pending_orders.values().cloned().collect();
        let to_cancel = self.pending_orders_to_cancel.values().cloned().collect();
        {
            let pending = std::mem::take(&mut self.pending_orders);
            self.orders.extend(pending.into_iter());
            let to_cancel = std::mem::take(&mut self.pending_orders_to_cancel);
            self.orders_to_cancel.extend(to_cancel.into_iter());
        }
        (orders, to_cancel)
    }
    pub fn remove_completed_order(&mut self, id: u64) {
        self.orders.remove(&id);
        self.orders_to_cancel.remove(&id);
        self.pending_orders_to_cancel.remove(&id);
    }
    pub fn try_cancel_order(&mut self, id: u64, user: Principal) {
        let info = self.orders.get(&id).expect("order not found").clone();
        assert_eq!(user, info.user, "invalid order id");
        self.pending_orders_to_cancel.insert(id, info);
    }
}
