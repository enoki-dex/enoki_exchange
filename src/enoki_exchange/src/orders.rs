use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Nat, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::types::*;

thread_local! {
    static STATE: RefCell<OrdersState> = RefCell::new(OrdersState::default());
}

#[derive(serde::Deserialize, serde::Serialize, CandidType, Clone, Debug)]
pub struct Order {
    broker: Principal
}

impl Default for Order {
    fn default() -> Self {
        Self {
            broker: Principal::anonymous()
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, CandidType, Clone, Debug, Default)]
pub struct OrdersState {
    cancelled_orders: HashMap<Principal, Vec<Order>>,
}

pub fn take_cancelled_orders() -> HashMap<Principal, Vec<Order>> {
    STATE.with(|s| std::mem::take(&mut s.borrow_mut().cancelled_orders))
}

pub fn match_orders(new_orders: Vec<Order>, orders_to_cancel: Vec<u64>) {
    todo!()
}