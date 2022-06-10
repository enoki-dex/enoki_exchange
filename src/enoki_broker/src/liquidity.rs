use std::borrow::BorrowMut;
use std::cell::{RefCell, RefMut};
use std::collections::{BTreeMap, HashMap};
use std::convert::TryInto;
use std::ops::{AddAssign, Div, Mul, Sub, SubAssign};

use candid::{candid_method, CandidType, Deserialize, Nat, Principal};
use ic_cdk_macros::*;
use serde::Serialize;

use enoki_exchange_shared::has_sharded_users::{get_user_shard, register_user};
use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::has_token_info::{
    get_assigned_shard, get_assigned_shards, price_in_b_float_to_u64, AssignedShards,
};
use enoki_exchange_shared::interfaces::enoki_wrapped_token::ShardedTransferNotification;
use enoki_exchange_shared::is_managed;
use enoki_exchange_shared::is_managed::{assert_is_manager, get_manager};
use enoki_exchange_shared::liquidity::liquidity_pool::LiquidityPool;
use enoki_exchange_shared::liquidity::{
    RequestForNewLiquidityTarget, ResponseAboutLiquidityChanges,
};
use enoki_exchange_shared::types::*;
use enoki_exchange_shared::utils::nat_to_u64;
use crate::payoffs;

thread_local! {
    static STATE: RefCell<LiquidityState> = RefCell::new(LiquidityState::default());
}

#[derive(Deserialize, CandidType, Clone, Debug, Default)]
struct LiquidityState {
    bid_ask: AggregateBidAsk,
    available_liquidity: LiquidityAmount,
    liquidity_traded: LiquidityTrades,
}

pub fn update_liquidity_target(
    bid_ask: AggregateBidAsk,
    target: RequestForNewLiquidityTarget,
) -> ResponseAboutLiquidityChanges {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.bid_ask = bid_ask;

        let removed = s.available_liquidity.sub_or_zero(&target.target);
        let mut added = target.target.sub_or_zero(&s.available_liquidity);
        added.token_a = added.token_a.min(target.extra_liquidity_available.token_a);
        added.token_b = added.token_b.min(target.extra_liquidity_available.token_b);

        s.available_liquidity.add_assign(added.clone());
        s.available_liquidity.sub_assign(removed.clone());

        ResponseAboutLiquidityChanges {
            added,
            removed,
            traded: s.liquidity_traded.clone(),
        }
    })
}

pub async fn swap(order: ProcessedOrderInput) {
    let swap: Result<LiquidityReference> = STATE.with(|s| {
        let mut s = s.borrow_mut();
        let avg_price = s.bid_ask.get_avg_price_for(order.side.clone(), order.quantity.clone())?;
        if match order.side {
            Side::Buy => avg_price > order.limit_price_in_b,
            Side::Sell => avg_price < order.limit_price_in_b,
        } {
            return Err(TxError::SlippageExceeded);
        }

        Ok(s.bid_ask.execute_swap(order.side.clone(), order.quantity.clone()))
    });
    let swap = swap.unwrap();
    let result = payoffs::exchange_swap(order, swap).await;

}

trait SwapLiquidity {
    fn get_avg_price_for(&self, action: Side, quantity: Nat) -> Result<u64>;
    fn execute_swap(&mut self, action: Side, quantity: Nat) -> LiquidityReference;
}

impl SwapLiquidity for AggregateBidAsk {
    fn get_avg_price_for(&self, action: Side, quantity: Nat) -> Result<u64> {
        let mut price_times_quantity = Nat::from(0u32);
        let mut quantity_remaining = quantity.clone();
        match action {
            Side::Buy => {
                for (price, party) in self
                    .asks
                    .iter()
                    .flat_map(|(&price, parties)| parties.into_iter().map(move |p| (price, p)))
                {
                    let diff = quantity_remaining.clone().min(party.quantity.0.clone());
                    quantity_remaining.sub_assign(diff.clone());
                    price_times_quantity.add_assign(diff * price);
                    if quantity_remaining == 0u32 {
                        break;
                    }
                }
            }
            Side::Sell => {
                for (price, party) in self
                    .bids
                    .iter()
                    .rev()
                    .flat_map(|(&price, parties)| parties.into_iter().map(move |p| (price, p)))
                {
                    let diff = quantity_remaining.clone().min(party.quantity.0.clone());
                    quantity_remaining.sub_assign(diff.clone());
                    price_times_quantity.add_assign(diff * price);
                    if quantity_remaining == 0u32 {
                        break;
                    }
                }
            }
        }
        if quantity_remaining != 0u32 {
            return Err(TxError::InsufficientLiquidityAvailable);
        }
        let avg_price = price_times_quantity / quantity;
        nat_to_u64(avg_price)
    }

    fn execute_swap(&mut self, action: Side, quantity: Nat) -> LiquidityReference {
        let mut quantity_remaining = quantity.clone();
        let mut liquidity_reference = LiquidityReference::default();
        match action {
            Side::Buy => {
                for (price, party) in self
                    .asks
                    .iter_mut()
                    .flat_map(|(&price, parties)| parties.into_iter().map(move |p| (price, p)))
                {
                    let diff = quantity_remaining.clone().min(party.quantity.0.clone());
                    quantity_remaining.sub_assign(diff.clone());
                    party.quantity.0.sub_assign(diff.clone());
                    let mut reference = party.clone();
                    reference.quantity = StableNat(diff);
                    liquidity_reference
                        .prices
                        .entry(price)
                        .or_default()
                        .push(reference);

                    if quantity_remaining == 0u32 {
                        break;
                    }
                }
            }
            Side::Sell => {
                for (price, party) in self
                    .bids
                    .iter_mut()
                    .rev()
                    .flat_map(|(&price, parties)| parties.into_iter().map(move |p| (price, p)))
                {
                    let diff = quantity_remaining.clone().min(party.quantity.0.clone());
                    quantity_remaining.sub_assign(diff.clone());
                    party.quantity.0.sub_assign(diff.clone());
                    let mut reference = party.clone();
                    reference.quantity = StableNat(diff);
                    liquidity_reference
                        .prices
                        .entry(price)
                        .or_default()
                        .push(reference);

                    if quantity_remaining == 0u32 {
                        break;
                    }
                }
            }
        }
        if quantity_remaining != 0u32 {
            panic!("Invalid operation: Should check price before executing swap");
        }

        //cleanup
        let target = match action {
            Side::Buy => &mut self.asks,
            Side::Sell => &mut self.bids,
        };
        for (&price, _) in liquidity_reference.prices.iter() {
            if let Some(parties) = target.get_mut(&price) {
                parties.retain(|p| p.quantity.is_nonzero());
                if parties.is_empty() {
                    target.remove(&price);
                }
            }
        }
        liquidity_reference
    }
}

#[derive(Default)]
pub struct LiquidityReference {
    prices: BTreeMap<u64, Vec<CounterpartyInfo>>,
}
