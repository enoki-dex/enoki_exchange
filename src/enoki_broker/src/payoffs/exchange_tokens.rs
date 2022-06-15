use candid::{Nat, Principal};
use futures::FutureExt;

use enoki_exchange_shared::has_sharded_users::get_user_shard;
use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::has_token_info::get_token_address;
use enoki_exchange_shared::has_trading_fees::get_limit_order_taker_fee;
use enoki_exchange_shared::types::*;
use enoki_exchange_shared::utils::nat_div_float;

use crate::payoffs::{
    with_failed_exchanges_mut, with_pending_transfers_mut,
    PendingTransfer, TokenExchangeInfo, TransferInfo, TransferPair,
};

async fn send_funds_from(id: String, broker: Principal, info: PendingTransfer, user_shard_id_to_retrieve: Principal) -> Result<()> {
    if broker == ic_cdk::id() {
        let shard_id_to_retrieve = get_user_shard(user_shard_id_to_retrieve, get_token_address(&info.token.opposite()))?;
        send_funds_internal(id, info, ic_cdk::id(), shard_id_to_retrieve).await
    } else {
        ic_cdk::println!("[broker] sending exchange id {} to broker {}", id, broker);
        ic_cdk::call(broker, "sendFunds", (id, info, user_shard_id_to_retrieve))
            .await
            .map_err(|e| e.into_tx_error())
    }
}

pub async fn send_funds_internal(
    id: String,
    info: PendingTransfer,
    notify_principal: Principal,
    shard_id_to_retrieve: Principal,
) -> Result<()> {
    let assigned_token_shard = has_token_info::get_assigned_shard(&info.token);
    let token_address = get_token_address(&info.token);
    let to_shard = get_user_shard(info.to, token_address)?;
    let message = format!("{}|{}", id, shard_id_to_retrieve.to_string());
    ic_cdk::println!("[broker] executing first half of exchange id {}", id);
    ic_cdk::call(
        assigned_token_shard,
        "shardTransferAndCall",
        (
            to_shard,
            info.to,
            info.amount,
            notify_principal,
            "fundsSent",
            message,
        ),
    )
    .await
    .map_err(|e| e.into_tx_error())
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
                            broker: market_maker.broker, // should be paid by
                            token: EnokiToken::TokenA,
                            to: order_info.user,
                            amount: market_maker.quantity.clone(),
                        },
                        other_user: TransferInfo {
                            broker: ic_cdk::id(), // should be paid by
                            token: EnokiToken::TokenB,
                            to: market_maker.user,
                            amount: plus_fees(has_token_info::quantity_a_to_b(
                                market_maker.quantity.clone().into(),
                                market_maker.price,
                            )?)?
                            .into(),
                        },
                    }),
                    Side::Sell => Ok(TokenExchangeInfo {
                        other_user: TransferInfo {
                            broker: ic_cdk::id(), // should be paid by
                            token: EnokiToken::TokenA,
                            to: market_maker.user,
                            amount: plus_fees(has_token_info::quantity_b_to_a(
                                market_maker.quantity.clone().into(),
                                market_maker.price,
                            )?)?
                            .into(),
                        },
                        local_user: TransferInfo {
                            broker: market_maker.broker, // should be paid by
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
                    ic_cdk::api::print(format!("[broker] error exchanging tokens: {:?}. Input: {:?}", err, exchange));
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
    ic_cdk::api::print(format!("[broker] executing token exchange: {:?}", exchange));

    let TokenExchangeInfo {
        local_user,
        other_user,
    } = exchange;

    let user_shard_id_to_retrieve = other_user.to;
    let id = with_pending_transfers_mut(|pending_transfers| {
        pending_transfers.create_new(TransferPair {
            waiting_on: local_user.clone(),
            next_transfer: other_user,
        })
    });

    send_funds_from(
        id.to_string(),
        local_user.broker,
        PendingTransfer {
            to: local_user.to,
            token: local_user.token,
            amount: local_user.amount.into(),
        },
        user_shard_id_to_retrieve
    )
    .await
}
