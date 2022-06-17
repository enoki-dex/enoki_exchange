use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};

use candid::{candid_method, CandidType, Deserialize, Nat, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_sharded_users::get_user_shard;
use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::has_token_info::AssignedShards;
use enoki_exchange_shared::interfaces::enoki_wrapped_token::ShardedTransferNotification;
use enoki_exchange_shared::types::*;
pub use exchange_tokens::exchange_tokens;
pub use fees::charge_deposit_fee;
pub use fees::{
    export_stable_storage as export_stable_storage_fees,
    import_stable_storage as import_stable_storage_fees, AccruedFees,
};
pub use market_maker_extra_rewards::{add_reward, distribute_market_maker_rewards};
pub use swap_tokens::send_swap_tokens;

use crate::other_brokers::assert_is_broker;
use crate::payoffs::market_maker_extra_rewards::MarketMakerAccruedExtraRewards;
use crate::payoffs::token_shard_validation::is_valid_token_shard;

mod exchange_tokens;
mod fees;
mod market_maker_extra_rewards;
mod swap_tokens;
mod token_shard_validation;

thread_local! {
    static STATE: RefCell<PayoffsState> = RefCell::new(PayoffsState::default());
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct PayoffsState {
    pending_transfers: PendingTransfers,
    failed_exchanges: Vec<TokenExchangeInfo>,
    broker_assigned_shards: HashMap<(Principal, EnokiToken), Principal>,
    valid_token_shards_a: HashSet<Principal>,
    valid_token_shards_b: HashSet<Principal>,
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
    pub fn get_token_for_first_part(&self, id: u64) -> Option<EnokiToken> {
        self.pending.get(&id).map(|val| val.waiting_on.token.clone())
    }
}

#[derive(Deserialize, CandidType, Clone, Debug)]
pub struct FirstTransfer {
    to: Principal,
    to_shard: Principal,
    token: EnokiToken,
    amount: Nat,
    user_for_shard_id_to_retrieve: Principal,
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug)]
pub struct TransferPair {
    waiting_on: TransferInfo,
    next_transfer: TransferInfo,
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone)]
pub struct TransferInfo {
    broker: Principal,
    token: EnokiToken,
    to: Principal,
    amount: StableNat,
}

impl Debug for TransferInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let TransferInfo {
            broker,
            token,
            to,
            amount,
        } = self;
        write!(
            f,
            "TransferInfo {{ broker: {}, token: {:?}, to: {}, amount: {:?} }}",
            broker, token, to, amount
        )
    }
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug)]
pub struct TokenExchangeInfo {
    local_user: TransferInfo,
    other_user: TransferInfo,
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

fn with_valid_token_shards<F: FnOnce(&mut HashSet<Principal>) -> R, R>(
    token: &EnokiToken,
    f: F,
) -> R {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        match token {
            EnokiToken::TokenA => f(&mut s.valid_token_shards_a),
            EnokiToken::TokenB => f(&mut s.valid_token_shards_b),
        }
    })
}

fn with_pending_market_maker_rewards<F: FnOnce(&mut MarketMakerAccruedExtraRewards) -> R, R>(
    f: F,
) -> R {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        f(&mut s.market_maker_pending_rewards)
    })
}

async fn get_broker_assigned_shard(broker: Principal, token: EnokiToken) -> Result<Principal> {
    let key = (broker, token.clone());
    if let Some(shard) = STATE.with(|s| s.borrow().broker_assigned_shards.get(&key).copied()) {
        return Ok(shard);
    }
    let shard = if broker == ic_cdk::id() {
        let assigned = has_token_info::get_assigned_shard(&token);
        STATE.with(|s| s.borrow_mut().broker_assigned_shards.insert(key, assigned));
        assigned
    } else {
        let result: Result<(AssignedShards,)> = ic_cdk::call(broker, "getAssignedShards", ())
            .await
            .map_err(|e| e.into_tx_error());
        let AssignedShards { token_a, token_b } = result?.0;
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            s.broker_assigned_shards
                .insert((broker, EnokiToken::TokenA), token_a);
            s.broker_assigned_shards
                .insert((broker, EnokiToken::TokenB), token_b);
        });
        match &token {
            EnokiToken::TokenA => token_a,
            EnokiToken::TokenB => token_b,
        }
    };
    Ok(shard)
}

#[update(name = "sendFunds")]
#[candid_method(update, rename = "sendFunds")]
async fn send_funds(id: String, info: FirstTransfer) {
    assert_is_broker(ic_cdk::caller()).unwrap();
    ic_cdk::println!(
        "[broker] received exchange id {} from broker {}",
        id,
        ic_cdk::caller()
    );
    let shard_id_to_retrieve = get_user_shard(
        info.user_for_shard_id_to_retrieve,
        has_token_info::get_token_address(&info.token.opposite()),
    )
    .unwrap();
    exchange_tokens::send_funds_internal(id, info, ic_cdk::caller(), shard_id_to_retrieve)
        .await
        .unwrap()
}

#[update(name = "fundsSent")]
#[candid_method(update, rename = "fundsSent")]
async fn funds_sent(notification: ShardedTransferNotification) -> String {
    let mut data = notification.data.split('|');
    let id: u64 = data
        .next()
        .expect("invalid message body fundsSent")
        .parse()
        .expect("cannot parse id");
    let user_shard_to_retrieve: Principal = data
        .next()
        .expect("invalid message body fundsSent")
        .parse()
        .expect("cannot parse user_shard_to_retrieve");
    if !is_valid_token_shard(
        &STATE
            .with(|s| s.borrow().pending_transfers.get_token_for_first_part(id))
            .expect("id not found"),
        ic_cdk::caller(),
    )
    .await
    {
        panic!("Unauthorized notification from {}", ic_cdk::caller());
    }
    let TransferPair {
        waiting_on,
        next_transfer,
    } = STATE
        .with(|s| s.borrow_mut().pending_transfers.remove(id))
        .expect("cannot find id");
    assert_eq!(
        waiting_on.to, notification.to,
        "recipient not the same as expected"
    );
    assert_eq!(
        waiting_on.amount.to_nat(),
        notification.value + notification.fee_charged,
        "amount received not the same as expected"
    );

    let assigned_token_shard = has_token_info::get_assigned_shard(&next_transfer.token);

    ic_cdk::println!(
        "[broker] executing second half of exchange id {}",
        notification.data
    );

    let response: Result<()> = ic_cdk::call(
        assigned_token_shard,
        "shardTransfer",
        (
            user_shard_to_retrieve,
            next_transfer.to,
            next_transfer.amount.to_nat(),
        ),
    )
    .await
    .map_err(|e| e.into_tx_error());
    response.unwrap();
    "OK".to_string()
}

pub fn export_stable_storage() -> PayoffsState {
    let data = STATE.with(|s| s.take());
    data
}

pub fn import_stable_storage(data: PayoffsState) {
    STATE.with(|s| s.replace(data));
}
