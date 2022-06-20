use std::cell::RefCell;

use candid::{candid_method, CandidType, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_sharded_users::register_user;
use enoki_exchange_shared::interfaces::enoki_wrapped_token::ShardedTransferNotification;
use enoki_exchange_shared::is_managed::assert_is_manager;
use enoki_exchange_shared::liquidity::{
    RequestForNewLiquidityTarget, ResponseAboutLiquidityChanges,
};
use enoki_exchange_shared::types::*;
use enoki_exchange_shared::{has_sharded_users, has_token_info};

use crate::orders::order_book::OrderBook;
use crate::orders::order_history::OrderHistory;
use crate::payoffs::distribute_market_maker_rewards;
use crate::users::assert_is_user;
use crate::{liquidity, payoffs};

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
fn submit_limit_order(notification: ShardedTransferNotification) -> String {
    let input = order_parser::validate_order_input(notification, false).unwrap();
    assert_is_user(input.user).unwrap();
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let (user, id) = s.order_book.create_limit_order(input);
        s.order_history.add_new_order(user, id);
        id.to_string()
    })
}

#[update(name = "swap")]
#[candid_method(update)]
async fn swap(notification: ShardedTransferNotification) -> String {
    let input = order_parser::validate_order_input(notification, true).unwrap();
    liquidity::swap(input).await;
    "OK".to_string()
}

#[update(name = "cancelOrder")]
#[candid_method(update, rename = "cancelOrder")]
fn cancel_order(order_id: u64) {
    let from = ic_cdk::caller();
    STATE.with(|s| s.borrow_mut().order_book.try_cancel_order(order_id, from));
}

#[update(name = "cancelAllOpenOrders")]
#[candid_method(update, rename = "cancelAllOpenOrders")]
fn cancel_all_open_orders() {
    let from = ic_cdk::caller();
    let orders = get_open_orders(from);
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        for order in orders.open_orders {
            s.order_book.try_cancel_order(order.id, from);
        }
    });
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

#[query(name = "getOpenOrdersCount")]
#[candid_method(query, rename = "getOpenOrdersCount")]
fn get_open_orders_count() -> usize {
    STATE.with(|s| {
        let s = s.borrow();
        s.order_history.get_open_orders_count()
    })
}

#[query(name = "getPastOrders")]
#[candid_method(query, rename = "getPastOrders")]
fn get_past_orders(user: Principal) -> Vec<OrderShare> {
    STATE.with(|s| {
        s.borrow()
            .order_history
            .get_past_orders(user)
            .into_iter()
            .map(|o| o.into())
            .collect()
    })
}

#[query(name = "getAccruedExtraRewards")]
#[candid_method(query, rename = "getAccruedExtraRewards")]
fn get_accrued_extra_rewards(user: Principal) -> LiquidityAmountNat {
    STATE.with(|s| {
        s.borrow()
            .order_history
            .get_accrued_extra_rewards(user)
            .into()
    })
}

pub fn add_accrued_extra_reward(user: Principal, amount: StableNat, token: &EnokiToken) {
    STATE.with(|s| {
        s.borrow_mut()
            .order_history
            .add_accrued_extra_reward(user, amount, token)
    });
}

#[query(name = "isUserRegistered")]
#[candid_method(query, rename = "isUserRegistered")]
pub fn is_user_registered(user: Principal) -> bool {
    has_sharded_users::get_user_shard(user, has_token_info::get_token_address(&EnokiToken::TokenA))
        .is_ok()
        && has_sharded_users::get_user_shard(
            user,
            has_token_info::get_token_address(&EnokiToken::TokenB),
        )
        .is_ok()
}

#[update(name = "register")]
#[candid_method(update)]
async fn register(user: Principal) {
    register_user(user).await.unwrap();
}

pub fn export_stable_storage() -> OrdersState {
    STATE.with(|s| s.take())
}

pub fn import_stable_storage(data: OrdersState) {
    STATE.with(|s| s.replace(data));
}
