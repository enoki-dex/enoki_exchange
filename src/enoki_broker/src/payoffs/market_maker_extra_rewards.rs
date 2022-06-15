use std::collections::HashMap;
use std::ops::AddAssign;

use candid::{candid_method, CandidType, Nat, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_sharded_users::get_user_shard;
use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::interfaces::enoki_wrapped_token::ShardedTransferNotification;
use enoki_exchange_shared::types::*;

use crate::orders::add_accrued_extra_reward;
use crate::other_brokers::assert_is_broker;
use crate::payoffs;
use crate::payoffs::fees::use_fee_for_transfer;
use crate::payoffs::{fees, with_pending_market_maker_rewards};

const MIN_AMOUNT_TO_SEND_WITH_RESPECT_TO_FEE: u64 = 10;

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
        user_rewards.0.values().cloned().sum::<StableNat>().to_nat(),
        notification.value
    );
    for (user, reward) in user_rewards.0 {
        add_reward(ic_cdk::id(), user, &token, reward.into());
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
    let transfer_fee_a = if let Some(fee) = fees::try_get_fee_for_transfer(&EnokiToken::TokenA) {
        fee
    } else {
        fees::get_fee_for_transfer(&EnokiToken::TokenA)
            .await
            .unwrap()
    };
    let transfer_fee_b = if let Some(fee) = fees::try_get_fee_for_transfer(&EnokiToken::TokenB) {
        fee
    } else {
        fees::get_fee_for_transfer(&EnokiToken::TokenB)
            .await
            .unwrap()
    };
    distribute_local_rewards(transfer_fee_a.clone(), transfer_fee_b.clone()).await;
    distribute_other_broker_rewards(transfer_fee_a, transfer_fee_b).await;
}

async fn distribute_other_broker_rewards(transfer_fee_a: Nat, transfer_fee_b: Nat) {
    let rewards = with_pending_market_maker_rewards(|rewards| {
        std::mem::take(&mut rewards.other_broker_rewards)
    });
    let mut failed: HashMap<Principal, HashMap<Principal, LiquidityAmount>> = HashMap::new();

    for token in [EnokiToken::TokenA, EnokiToken::TokenB] {
        let shard_address = has_token_info::get_assigned_shard(&token);
        let transfer_fee = match token {
            EnokiToken::TokenA => transfer_fee_a.clone(),
            EnokiToken::TokenB => transfer_fee_b.clone(),
        };
        for (&broker, reward) in rewards.iter() {
            async fn transfer_to_broker(
                shard_address: Principal,
                broker: Principal,
                token: &EnokiToken,
                reward: &HashMap<Principal, LiquidityAmount>,
                transfer_fee: &Nat,
            ) -> Result<()> {
                let user_rewards = UserRewards::new(reward, &token);
                let rewards_total: StableNat = user_rewards.0.values().cloned().sum();
                let mut value: Nat = rewards_total.into();
                if transfer_fee.clone() * MIN_AMOUNT_TO_SEND_WITH_RESPECT_TO_FEE > value {
                    return Err(TxError::QuantityTooLow.into());
                }
                let fee = use_fee_for_transfer(&token).await?;
                value += fee;
                let user_rewards_str = serde_json::to_string(&user_rewards)
                    .map_err(|e| TxError::ParsingError(format!("{:?}", e)))?;
                let broker_shard = payoffs::get_broker_assigned_shard(broker, token.clone()).await?;
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
                .map_err(|e| e.into_tx_error());

                result
            }
            if let Err(error) =
                transfer_to_broker(shard_address, broker, &token, reward, &transfer_fee).await
            {
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

async fn distribute_local_rewards(transfer_fee_a: Nat, transfer_fee_b: Nat) {
    let local_rewards =
        with_pending_market_maker_rewards(|rewards| std::mem::take(&mut rewards.local_rewards));
    let mut failed: HashMap<Principal, LiquidityAmount> = HashMap::new();
    for token in [EnokiToken::TokenA, EnokiToken::TokenB] {
        let transfer_fee = match token {
            EnokiToken::TokenA => transfer_fee_a.clone(),
            EnokiToken::TokenB => transfer_fee_b.clone(),
        };
        let token_address = has_token_info::get_token_address(&token);
        let shard_address = has_token_info::get_assigned_shard(&token);
        for (&user, reward) in local_rewards.iter() {
            let token_reward: Nat = reward.get(&token).clone().into();
            if transfer_fee.clone() * MIN_AMOUNT_TO_SEND_WITH_RESPECT_TO_FEE > token_reward {
                failed
                    .entry(user)
                    .or_default()
                    .get_mut(&token)
                    .add_assign(token_reward.into());
                continue;
            }
            match get_user_shard(user, token_address) {
                Ok(user_shard) => {
                    ic_cdk::api::print(format!(
                        "[broker] rewarding market maker {} with {:?} {:?}",
                        user,
                        token,
                        reward.get(&token)
                    ));
                    let result: Result<()> = ic_cdk::call(
                        shard_address,
                        "shardTransfer",
                        (user_shard, user, token_reward.clone()),
                    )
                    .await
                    .map_err(|e| e.into_tx_error());
                    if let Err(err) = result {
                        ic_cdk::api::print(format!(
                            "[broker] error distributing extra reward: {:?}",
                            err
                        ));
                        failed
                            .entry(user)
                            .or_default()
                            .get_mut(&token)
                            .add_assign(token_reward.into());
                    } else {
                        add_accrued_extra_reward(user, token_reward.clone().into(), &token);
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
