use std::borrow::BorrowMut;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::convert::TryInto;
use std::ops::{AddAssign, Div, Mul, Sub, SubAssign};

use candid::parser::token::Token;
use candid::{candid_method, CandidType, Deserialize, Nat, Principal};
use futures::FutureExt;
use ic_cdk_macros::*;

use enoki_exchange_shared::has_sharded_users::{get_user_shard, register_user};
use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::has_token_info::{
    get_assigned_shard, get_assigned_shards, get_token_address, price_in_b_float_to_u64,
    AssignedShards,
};
use enoki_exchange_shared::has_trading_fees::get_limit_order_taker_fee;
use enoki_exchange_shared::interfaces::enoki_wrapped_token::ShardedTransferNotification;
use enoki_exchange_shared::is_managed;
use enoki_exchange_shared::is_managed::{assert_is_manager, get_manager};
use enoki_exchange_shared::liquidity::liquidity_pool::LiquidityPool;
use enoki_exchange_shared::liquidity::{
    RequestForNewLiquidityTarget, ResponseAboutLiquidityChanges,
};
use enoki_exchange_shared::types::*;
use enoki_exchange_shared::utils::nat_div_float;

use crate::liquidity::LiquidityReference;
use crate::other_brokers::assert_is_broker;
use crate::payoffs::{
    get_broker_assigned_shard, with_failed_exchanges_mut, with_pending_transfers_mut,
    PendingTransfer, TokenExchangeInfo, TransferInfo, TransferPair,
};

async fn send_funds_from(id: String, broker: Principal, info: PendingTransfer) -> Result<()> {
    if broker == ic_cdk::id() {
        send_funds_internal(id, info).await
    } else {
        ic_cdk::call(broker, "sendFunds", (id, info))
            .await
            .map_err(|e| e.into())
    }
}

pub async fn send_funds_internal(id: String, info: PendingTransfer) -> Result<()> {
    let assigned_token_shard = get_assigned_shard(&info.token);
    let token_address = get_token_address(&info.token);
    let to_shard = get_user_shard(info.to, token_address)?;
    ic_cdk::call(
        assigned_token_shard,
        "shardTransferAndCall",
        (
            to_shard,
            info.to,
            info.amount,
            ic_cdk::caller(),
            "fundsSent",
            id,
        ),
    )
    .await
    .map_err(|e| e.into())
}
pub fn exchange_tokens(orders: Vec<Order>) -> Vec<Order> {
    let mut failed_orders: Vec<Order> = Vec::new();
    let plus_fees = |val: Nat| -> Result<Nat> {
        let taker_fee = get_limit_order_taker_fee();
        nat_div_float(val, 1.0 - taker_fee)
    };
    let exchanges: Vec<TokenExchangeInfo> = orders
        .into_iter()
        .filter_map(|order| {
            let order_info = order.info.clone();
            order
                .clone()
                .state
                .marker_makers
                .into_iter()
                .map(move |market_maker| match &order_info.side {
                    Side::Buy => Ok(TokenExchangeInfo {
                        local_user: TransferInfo {
                            broker: ic_cdk::id(),
                            token: EnokiToken::TokenA,
                            to: order_info.user,
                            amount: market_maker.quantity.clone(),
                        },
                        other_user: TransferInfo {
                            broker: market_maker.broker,
                            token: EnokiToken::TokenB,
                            to: market_maker.user,
                            amount: plus_fees(has_token_info::quant_a_to_quant_b(
                                market_maker.quantity.0.clone(),
                                market_maker.price,
                            )?)?.into(),
                        },
                    }),
                    Side::Sell => Ok(TokenExchangeInfo {
                        other_user: TransferInfo {
                            broker: market_maker.broker,
                            token: EnokiToken::TokenA,
                            to: market_maker.user,
                            amount: plus_fees(has_token_info::quant_b_to_quant_a(
                                market_maker.quantity.0.clone(),
                                market_maker.price,
                            )?)?.into(),
                        },
                        local_user: TransferInfo {
                            broker: ic_cdk::id(),
                            token: EnokiToken::TokenB,
                            to: order_info.user,
                            amount: market_maker.quantity.clone(),
                        },
                    }),
                })
                .collect::<Result<Vec<TokenExchangeInfo>>>()
                .map_err(|_| failed_orders.push(order))
                .ok()
        })
        .flat_map(|order| order)
        .collect();
    ic_cdk::spawn(execute_exchanges(exchanges));
    failed_orders
}

async fn execute_exchanges(mut exchanges: Vec<TokenExchangeInfo>) {
    let mut older_pending = with_failed_exchanges_mut(|failed| std::mem::take(failed));
    exchanges.append(&mut older_pending);

    let results: Vec<Option<TokenExchangeInfo>> =
        futures::future::join_all(exchanges.into_iter().map(|exchange| {
            execute_exchange(exchange.clone()).map(|res: Result<()>| {
                if let Err(err) = res {
                    ic_cdk::api::print(format!("error exchanging tokens: {:?}", err));
                    Some(exchange)
                } else {
                    None
                }
            })
        }))
        .await;
    let mut failed: Vec<_> = results.into_iter().filter_map(|r| r).collect();
    if !failed.is_empty() {
        with_failed_exchanges_mut(|f| f.append(&mut failed));
    }
}

async fn execute_exchange(exchange: TokenExchangeInfo) -> Result<()> {
    let TokenExchangeInfo {
        local_user,
        other_user,
    } = exchange;

    get_broker_assigned_shard(other_user.broker, other_user.token.clone()).await?;
    let id = with_pending_transfers_mut(|pending_transfers| {
        pending_transfers.create_new(TransferPair {
            waiting_on: other_user.clone(),
            next_transfer: local_user,
        })
    });

    send_funds_from(
        id.to_string(),
        other_user.broker,
        PendingTransfer {
            to: other_user.to,
            token: other_user.token,
            amount: other_user.amount.into(),
        },
    )
    .await
}
