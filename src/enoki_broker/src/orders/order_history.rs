use std::collections::HashMap;
use std::ops::AddAssign;

use candid::{CandidType, Principal};

use enoki_exchange_shared::types::*;

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct OrderHistory {
    current_orders: HashMap<Principal, Vec<u64>>,
    past_orders: HashMap<Principal, HashMap<u64, Order>>,
    accrued_extra_rewards: HashMap<Principal, LiquidityAmount>,
}

impl OrderHistory {
    pub fn add_new_order(&mut self, user: Principal, order_id: u64) {
        self.current_orders.entry(user).or_default().push(order_id);
    }
    pub fn add_completed_order(&mut self, order: Order) {
        self.current_orders
            .get_mut(&order.info.user)
            .map(|user_orders| user_orders.retain(|&o| o != order.info.id));
        self.past_orders.entry(order.info.user).or_default().insert(order.info.id, order);
    }
    pub fn add_accrued_extra_reward(&mut self, user: Principal, amount: StableNat, token: &EnokiToken) {
        self.accrued_extra_rewards.entry(user).or_default().get_mut(token).add_assign(amount);
    }
    pub fn get_open_orders(&self, user: Principal) -> Vec<u64> {
        self.current_orders.get(&user).cloned().unwrap_or_default()
    }
    pub fn get_past_orders(&self, user: Principal) -> Vec<Order> {
        self.past_orders.get(&user).map(|past| past.values().cloned().collect()).unwrap_or_default()
    }
    pub fn get_accrued_extra_rewards(&self, user: Principal) -> LiquidityAmount {
        self.accrued_extra_rewards.get(&user).cloned().unwrap_or_default()
    }
}
