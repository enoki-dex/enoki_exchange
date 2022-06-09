use std::borrow::BorrowMut;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::convert::TryInto;
use std::ops::{AddAssign, Div, Mul, Sub, SubAssign};

use candid::{candid_method, CandidType, Deserialize, Nat, Principal};
use ic_cdk_macros::*;
use serde::Serialize;

use enoki_exchange_shared::has_sharded_users::{get_user_shard, register_user};
use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::has_token_info::{
    get_assigned_shard, get_assigned_shards, AssignedShards,
};
use enoki_exchange_shared::interfaces::enoki_wrapped_token::ShardedTransferNotification;
use enoki_exchange_shared::is_managed;
use enoki_exchange_shared::is_managed::{assert_is_manager, get_manager};
use enoki_exchange_shared::liquidity::liquidity_pool::LiquidityPool;
use enoki_exchange_shared::liquidity::{
    RequestForNewLiquidityTarget, ResponseAboutLiquidityChanges,
};
use enoki_exchange_shared::types::*;
use enoki_exchange_shared::utils::nat_x_float;

use crate::orders::order_book::OrderBook;
use crate::orders::order_history::OrderHistory;

mod order_book;
mod order_history;

thread_local! {
    static STATE: RefCell<OrdersState> = RefCell::new(OrdersState::default());
}

#[derive(Deserialize, CandidType, Clone, Debug, Default)]
struct OrdersState {
    order_book: OrderBook,
    order_history: OrderHistory,
}

#[update(name = "retrieveOrders")]
#[candid_method(update, rename = "retrieveOrders")]
fn retrieve_orders() -> (Vec<OrderInfo>, Vec<OrderInfo>) {
    assert_is_manager().unwrap();

    todo!()
}

#[update(name = "submitCompletedOrders")]
#[candid_method(update, rename = "submitCompletedOrders")]
fn submit_completed_orders(
    completed: Vec<Order>,
    aggregate_bid_ask: AggregateBidAsk,
    request: RequestForNewLiquidityTarget,
) -> ResponseAboutLiquidityChanges {
    assert_is_manager().unwrap();
    todo!()
}

fn validate_order_input(
    notification: ShardedTransferNotification,
    is_swap: bool,
) -> Result<ProcessedOrderInput> {
    let token = has_token_info::parse_from()?;
    let user = notification.from;
    let order: OrderInput = serde_json::from_str(&notification.data)
        .map_err(|e| TxError::ParsingError(e.to_string()))?;
    let (side, quantity) = match &token {
        EnokiToken::TokenA => (
            Side::Sell,
            nat_x_float(notification.value, order.limit_price_in_b)?,
        ),
        EnokiToken::TokenB => (Side::Buy, notification.value),
    };
    register_user(
        user,
        has_token_info::get_token_address(&token),
        notification.from_shard,
    );

    Ok(ProcessedOrderInput {
        user,
        side,
        quantity,
        maker_taker: match (is_swap, order.allow_taker) {
            (true, _) => MakerTaker::OnlyTaker,
            (false, true) => MakerTaker::MakerOrTaker,
            (false, false) => MakerTaker::OnlyMaker,
        },
        limit_price_in_b: order.limit_price_in_b,
        expiration_time: order.expiration_time,
    })
}

#[update(name = "limitOrder")]
#[candid_method(update, rename = "limitOrder")]
fn submit_limit_order(notification: ShardedTransferNotification) {
    let input = validate_order_input(notification, false).unwrap();
    let order_id = STATE.with(|s| s.borrow_mut().order_book.create_limit_order(input).unwrap());
    todo!()
}

#[update(name = "swap")]
#[candid_method(update)]
fn swap(notification: ShardedTransferNotification) {
    let input = validate_order_input(notification, true).unwrap();
    todo!()
}
