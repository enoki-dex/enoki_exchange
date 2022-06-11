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
use crate::payoffs::fees::use_fee_for_transfer;
use crate::payoffs::{
    get_assigned_shard_for_broker, get_broker_assigned_shard, with_pending_market_maker_rewards,
};

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct MarketMakerAccruedExtraRewards {
    local_rewards: HashMap<Principal, LiquidityAmount>,
    other_broker_rewards: HashMap<Principal, HashMap<Principal, LiquidityAmount>>,
}

pub fn add_reward(broker: Principal, user: Principal, token: &EnokiToken, amount: Nat) {
    with_pending_market_maker_rewards(|rewards| {
        if broker == ic_cdk::id() {
            rewards
                .local_rewards
                .entry(user)
                .or_default()
                .get_mut(token)
                .add_assign(amount.into());
        } else {
            rewards
                .other_broker_rewards
                .entry(broker)
                .or_default()
                .entry(user)
                .or_default()
                .get_mut(token)
                .add_assign(amount.into());
        }
    })
}

#[update(name = "receiveMarketMakerRewards")]
#[candid_method(update, rename = "receiveMarketMakerRewards")]
fn receive_market_maker_rewards(notification: ShardedTransferNotification) {
    let token = has_token_info::parse_from().unwrap();
    let broker = notification.from;
    assert_is_broker(broker).unwrap();
    let user_rewards: UserRewards = serde_json::from_str(&notification.data).unwrap();
    assert_eq!(
        user_rewards.0.values().cloned().sum::<StableNat>().0,
        notification.value
    );
    for (user, reward) in user_rewards.0 {
        add_reward(ic_cdk::id(), user, &token, reward.0);
    }
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct UserRewards(HashMap<Principal, StableNat>);

impl UserRewards {
    pub fn new(rewards: &HashMap<Principal, LiquidityAmount>, token: &EnokiToken) -> Self {
        Self(
            rewards
                .iter()
                .map(|(&user, reward)| (user, reward.get(token).clone()))
                .collect(),
        )
    }
}

pub async fn distribute_market_maker_rewards() {
    distribute_local_rewards().await;
}

async fn distribute_other_broker_rewards() {
    let rewards = with_pending_market_maker_rewards(|rewards| {
        std::mem::take(&mut rewards.other_broker_rewards)
    });
    let mut failed: HashMap<Principal, HashMap<Principal, LiquidityAmount>> = HashMap::new();

    for token in [EnokiToken::TokenA, EnokiToken::TokenB] {
        let shard_address = get_assigned_shard(&token);
        for (&broker, reward) in rewards.iter() {
            async fn transfer_to_broker(
                shard_address: Principal,
                broker: Principal,
                token: &EnokiToken,
                reward: &HashMap<Principal, LiquidityAmount>,
            ) -> Result<()> {
                let fee = use_fee_for_transfer(&token).await?;
                let user_rewards = UserRewards::new(reward, &token);
                let rewards_total: StableNat = user_rewards.0.values().cloned().sum();
                let mut value: Nat = rewards_total.into();
                value += fee;
                let user_rewards_str = serde_json::to_string(&user_rewards)
                    .map_err(|e| TxError::ParsingError(format!("{:?}", e)))?;
                let broker_shard = get_broker_assigned_shard(broker, token.clone()).await?;
                let result: Result<()> = ic_cdk::call(
                    shard_address,
                    "shardTransferAndCall",
                    (
                        broker_shard,
                        broker,
                        value,
                        broker,
                        "receiveMarketMakerRewards",
                        user_rewards_str,
                    ),
                )
                .await
                .map_err(|e| e.into());

                result
            }
            if let Err(error) = transfer_to_broker(shard_address, broker, &token, reward).await {
                ic_cdk::api::print(format!(
                    "could not transfer market maker rewards to other broker: {:?}",
                    error
                ));
                for (&user, reward) in reward {
                    failed
                        .entry(broker)
                        .or_default()
                        .entry(user)
                        .or_default()
                        .get_mut(&token)
                        .add_assign(reward.get(&token).clone());
                }
            }
        }
    }

    with_pending_market_maker_rewards(|rewards| {
        for (broker, broker_rewards) in failed {
            let rewards = rewards.other_broker_rewards.entry(broker).or_default();
            for (user, user_reward) in broker_rewards {
                rewards.entry(user).or_default().add_assign(user_reward);
            }
        }
    })
}

async fn distribute_local_rewards() {
    let local_rewards =
        with_pending_market_maker_rewards(|rewards| std::mem::take(&mut rewards.local_rewards));
    let mut failed: HashMap<Principal, LiquidityAmount> = HashMap::new();
    for token in [EnokiToken::TokenA, EnokiToken::TokenB] {
        let token_address = get_token_address(&token);
        let shard_address = get_assigned_shard(&token);
        for (&user, reward) in local_rewards.iter() {
            let token_reward: Nat = reward.get(&token).clone().into();
            match get_user_shard(user, token_address) {
                Ok(user_shard) => {
                    let result: Result<()> = ic_cdk::call(
                        shard_address,
                        "shardTransfer",
                        (user_shard, user, token_reward.clone()),
                    )
                    .await
                    .map_err(|e| e.into());
                    if let Err(err) = result {
                        ic_cdk::api::print(format!("error distributing extra reward: {:?}", err));
                        failed
                            .entry(user)
                            .or_default()
                            .get_mut(&token)
                            .add_assign(token_reward.into());
                    }
                }
                Err(_) => {
                    failed
                        .entry(user)
                        .or_default()
                        .get_mut(&token)
                        .add_assign(token_reward.into());
                }
            }
        }
    }
    with_pending_market_maker_rewards(|rewards| {
        for (user, amount) in failed {
            rewards
                .local_rewards
                .entry(user)
                .or_default()
                .add_assign(amount);
        }
    });
}
