use std::cell::RefCell;

use candid::{candid_method, CandidType, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::interfaces::enoki_wrapped_token::ShardedTransferNotification;
use enoki_exchange_shared::is_managed::assert_is_manager;
use enoki_exchange_shared::liquidity::{
    RequestForNewLiquidityTarget, ResponseAboutLiquidityChanges,
};
use enoki_exchange_shared::types::*;

use crate::{liquidity, payoffs};
use crate::orders::order_book::OrderBook;
use crate::orders::order_history::OrderHistory;
use crate::payoffs::distribute_market_maker_rewards;

mod order_book;
mod order_history;
mod order_parser;

thread_local! {
    static STATE: RefCell<OrdersState> = RefCell::new(OrdersState::default());
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct OrdersState {
    order_book: OrderBook,
    order_history: OrderHistory,
    failed_orders: Vec<Order>,
}

#[update(name = "retrieveOrders")]
#[candid_method(update, rename = "retrieveOrders")]
fn retrieve_orders() -> (Vec<OrderInfo>, Vec<OrderInfo>) {
    assert_is_manager().unwrap();
    STATE.with(|s| s.borrow_mut().order_book.lock_pending_orders())
}

#[query(name = "getFailedOrders")]
#[candid_method(query, rename = "getFailedOrders")]
fn get_failed_orders() -> Vec<Order> {
    STATE.with(|s| s.borrow().failed_orders.clone())
}

#[update(name = "submitCompletedOrders")]
#[candid_method(update, rename = "submitCompletedOrders")]
fn submit_completed_orders(
    completed: Vec<Order>,
    aggregate_bid_ask: AggregateBidAsk,
    request: RequestForNewLiquidityTarget,
) -> ResponseAboutLiquidityChanges {
    assert_is_manager().unwrap();
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        for order in completed.iter() {
            s.order_book.remove_completed_order(order.info.id);
            s.order_history.add_completed_order(order.clone());
        }
    });
    let response = liquidity::update_liquidity_target(aggregate_bid_ask, request);
    resolve_completed_orders(completed);
    ic_cdk::spawn(distribute_market_maker_rewards());
    response
}

fn resolve_completed_orders(mut orders: Vec<Order>) {
    let mut older_orders = STATE.with(|s| std::mem::take(&mut s.borrow_mut().failed_orders));
    orders.append(&mut older_orders);
    let failed = payoffs::exchange_tokens(orders);
    STATE.with(|s| s.borrow_mut().failed_orders = failed);
}

#[update(name = "limitOrder")]
#[candid_method(update, rename = "limitOrder")]
fn submit_limit_order(notification: ShardedTransferNotification) {
    let input = order_parser::validate_order_input(notification, false).unwrap();
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let (user, id) = s.order_book.create_limit_order(input);
        s.order_history.add_new_order(user, id);
    })
}

#[update(name = "swap")]
#[candid_method(update)]
async fn swap(notification: ShardedTransferNotification) {
    let input = order_parser::validate_order_input(notification, true).unwrap();
    liquidity::swap(input).await;
}

#[query(name = "getOpenOrders")]
#[candid_method(query, rename = "getOpenOrders")]
fn get_open_orders(user: Principal) -> OpenOrderStatus {
    STATE.with(|s| {
        let s = s.borrow();
        let ids = s.order_history.get_open_orders(user);
        s.order_book.get_open_orders(&ids)
    })
}

#[query(name = "getPastOrders")]
#[candid_method(query, rename = "getPastOrders")]
fn get_past_orders(user: Principal) -> Vec<Order> {
    STATE.with(|s| s.borrow().order_history.get_past_orders(user))
}

pub fn export_stable_storage() -> OrdersState {
    STATE.with(|s| s.take())
}

pub fn import_stable_storage(data: OrdersState) {
    STATE.with(|s| s.replace(data));
}