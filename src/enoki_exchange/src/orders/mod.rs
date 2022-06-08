use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Nat, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::types::*;

use crate::orders::order_matcher::OrderMatcher;

mod order_matcher;
mod matching;

thread_local! {
    static STATE: RefCell<OrdersState> = RefCell::new(OrdersState::default());
}

#[derive(serde::Deserialize, serde::Serialize, CandidType, Clone, Debug, Default)]
pub struct OrdersState {
    orders: OrderMatcher,
}

pub fn match_orders(
    new_orders: Vec<OrderInfo>,
    orders_to_cancel: Vec<OrderInfo>,
) -> (HashMap<Principal, Vec<Order>>, AggregateBidAsk) {
    STATE.with(|s| {
        s.borrow_mut()
            .orders
            .match_orders(new_orders, orders_to_cancel)
    })
}