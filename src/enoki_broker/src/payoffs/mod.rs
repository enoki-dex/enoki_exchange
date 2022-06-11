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
use enoki_exchange_shared::interfaces::enoki_wrapped_token::ShardedTransferNotification;
use enoki_exchange_shared::is_managed;
use enoki_exchange_shared::is_managed::{assert_is_manager, get_manager};
use enoki_exchange_shared::liquidity::liquidity_pool::LiquidityPool;
use enoki_exchange_shared::liquidity::{
    RequestForNewLiquidityTarget, ResponseAboutLiquidityChanges,
};
use enoki_exchange_shared::types::*;

use crate::liquidity::LiquidityReference;
use crate::other_brokers::assert_is_broker;

mod exchange_tokens;
mod swap_tokens;
mod fees;
mod market_maker_extra_rewards;

pub use exchange_tokens::exchange_tokens;
pub use swap_tokens::send_swap_tokens;
pub use fees::charge_deposit_fee;
pub use market_maker_extra_rewards::{add_reward, distribute_market_maker_rewards};
pub use fees::{AccruedFees, import_stable_storage as import_stable_storage_fees, export_stable_storage as export_stable_storage_fees};
use crate::payoffs::market_maker_extra_rewards::MarketMakerAccruedExtraRewards;

thread_local! {
    static STATE: RefCell<PayoffsState> = RefCell::new(PayoffsState::default());
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct PayoffsState {
    pending_transfers: PendingTransfers,
    failed_exchanges: Vec<TokenExchangeInfo>,
    broker_assigned_shards: HashMap<(Principal, EnokiToken), Principal>,
    market_maker_pending_rewards: MarketMakerAccruedExtraRewards,
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct PendingTransfers {
    last_id: u64,
    pending: HashMap<u64, TransferPair>,
}

impl PendingTransfers {
    pub fn create_new(&mut self, pair: TransferPair) -> u64 {
        self.last_id += 1;
        let id = self.last_id;
        self.pending.insert(id, pair);
        id
    }
    pub fn remove(&mut self, id: u64) -> Option<TransferPair> {
        self.pending.remove(&id)
    }
}

#[derive(Deserialize, CandidType, Clone, Debug)]
pub struct PendingTransfer {
    to: Principal,
    token: EnokiToken,
    amount: Nat,
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug)]
struct TransferPair {
    waiting_on: TransferInfo,
    next_transfer: TransferInfo,
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug)]
struct TransferInfo {
    broker: Principal,
    token: EnokiToken,
    to: Principal,
    amount: StableNat,
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug)]
struct TokenExchangeInfo {
    local_user: TransferInfo,
    other_user: TransferInfo,
}

async fn get_broker_assigned_shard(broker: Principal, token: EnokiToken) -> Result<Principal> {
    let key = (broker, token.clone());
    if let Some(shard) = STATE.with(|s| s.borrow().broker_assigned_shards.get(&key).copied()) {
        return Ok(shard);
    }
    let shard = if broker == ic_cdk::id() {
        get_assigned_shard(&token)
    } else {
        let result: Result<(Principal,)> = ic_cdk::call(broker, "getAssignedShard", ())
            .await
            .map_err(|e| e.into());
        result?.0
    };
    STATE.with(|s| s.borrow_mut().broker_assigned_shards.insert(key, shard));
    Ok(shard)
}

fn with_failed_exchanges_mut<F: FnOnce(&mut Vec<TokenExchangeInfo>) -> R, R>(f: F) -> R {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        f(&mut s.failed_exchanges)
    })
}

fn with_pending_transfers_mut<F: FnOnce(&mut PendingTransfers) -> R, R>(f: F) -> R {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        f(&mut s.pending_transfers)
    })
}

fn with_pending_market_maker_rewards<F: FnOnce(&mut MarketMakerAccruedExtraRewards) -> R, R>(f: F) -> R {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        f(&mut s.market_maker_pending_rewards)
    })
}

#[query(name = "getAssignedShard")]
#[candid_method(query, rename = "getAssignedShard")]
async fn get_assigned_shard_for_broker(token: EnokiToken) -> Principal {
    get_assigned_shard(&token)
}

#[update(name = "sendFunds")]
#[candid_method(update, rename = "sendFunds")]
async fn send_funds(id: String, info: PendingTransfer) -> Result<()> {
    assert_is_broker(ic_cdk::caller())?;
    exchange_tokens::send_funds_internal(id, info).await
}

#[update(name = "fundsSent")]
#[candid_method(update, rename = "fundsSent")]
async fn funds_sent(notification: ShardedTransferNotification) {
    if !STATE.with(|s| {
        s.borrow()
            .broker_assigned_shards
            .values()
            .any(|val| *val == ic_cdk::caller())
    }) {
        panic!("Unauthorized");
    }
    let TransferPair {
        waiting_on,
        next_transfer,
    } = STATE
        .with(|s| {
            s.borrow_mut()
                .pending_transfers
                .remove(notification.data.parse().expect("cannot parse id"))
        })
        .expect("cannot find id");
    assert_eq!(
        waiting_on.to, notification.to,
        "recipient not the same as expected"
    );
    assert_eq!(
        waiting_on.amount.0,
        notification.value + notification.fee_charged,
        "amount received not the same as expected"
    );

    let assigned_token_shard = get_assigned_shard(&next_transfer.token);
    let token_address = get_token_address(&next_transfer.token);
    let to_shard = get_user_shard(next_transfer.to, token_address).unwrap();
    let response: Result<()> = ic_cdk::call(
        assigned_token_shard,
        "shardTransfer",
        (to_shard, next_transfer.to, next_transfer.amount),
    )
        .await
        .map_err(|e| e.into());
    response.unwrap();
}


pub fn export_stable_storage() -> PayoffsState {
    let data = STATE.with(|s| s.take());
    data
}

pub fn import_stable_storage(data: PayoffsState) {
    STATE.with(|s| s.replace(data));
}
