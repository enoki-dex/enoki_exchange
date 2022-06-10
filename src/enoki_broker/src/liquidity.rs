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
    get_assigned_shard, get_assigned_shards, price_in_b_float_to_u64, quant_b_to_quant_a,
    AssignedShards, QuantityTranslator,
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
        let avg_price = s
            .bid_ask
            .get_avg_price_for(order.side.clone(), order.quantity.clone())?;
        if match order.side {
            Side::Buy => avg_price > order.limit_price_in_b,
            Side::Sell => avg_price < order.limit_price_in_b,
        } {
            return Err(TxError::SlippageExceeded);
        }

        Ok(s.bid_ask
            .execute_swap(order.side.clone(), order.quantity.clone()))
    });
    let swap = swap.unwrap();
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let mut traded = LiquidityTrades::default();
        let quantity_supplier: StableNat = swap
            .prices
            .iter()
            .flat_map(|(p, val)| val.iter().map(|info| info.quantity.clone()))
            .sum();
        let quantity_user = order.quantity.clone();
        let (token_supplier, token_user) = match &order.side {
            Side::Buy => (EnokiToken::TokenA, EnokiToken::TokenB),
            Side::Sell => (EnokiToken::TokenB, EnokiToken::TokenA),
        };
        traded
            .increased
            .get_mut(&token_user)
            .add_assign(StableNat(quantity_user));
        traded
            .decreased
            .get_mut(&token_supplier)
            .add_assign(quantity_supplier);
        s.available_liquidity.add_assign(traded.increased.clone());
        s.available_liquidity.sub_assign(traded.decreased.clone());
        s.liquidity_traded.add_assign(traded);
    });
    payoffs::exchange_swap(order, swap).await.unwrap();
}

trait SwapLiquidity {
    fn get_avg_price_for(&self, action: Side, quantity: Nat) -> Result<u64>;
    fn execute_swap(&mut self, action: Side, quantity: Nat) -> LiquidityReference;
}

fn trade(quantity_a: &mut Nat, quantity_b: &mut Nat, price: u64) -> Nat {
    let mut quantity_translator = QuantityTranslator::new(price, quantity_a);
    let quantity_b_traded = quantity_translator
        .get_quantity_b()
        .unwrap()
        .min(quantity_b.clone());
    *quantity_b -= quantity_b_traded.clone();
    quantity_translator
        .sub_assign(quantity_b_traded.clone())
        .unwrap();
    quantity_b_traded
}

impl SwapLiquidity for AggregateBidAsk {
    fn get_avg_price_for(&self, action: Side, quantity: Nat) -> Result<u64> {
        let mut quantity_b_traded_total = Nat::from(0u32);
        let mut price_times_quantity = Nat::from(0u32);
        let mut quantity_remaining = quantity.clone();
        match action {
            Side::Buy => {
                for (price, party) in self
                    .asks
                    .iter()
                    .flat_map(|(&price, parties)| parties.into_iter().map(move |p| (price, p)))
                {
                    let mut party_quantity = party.quantity.0.clone();
                    let quantity_b_traded =
                        trade(&mut party_quantity, &mut quantity_remaining, price);
                    price_times_quantity.add_assign(quantity_b_traded.clone() * price);
                    quantity_b_traded_total.add_assign(quantity_b_traded);
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
                    let mut party_quantity = party.quantity.0.clone();
                    let quantity_b_traded =
                        trade(&mut quantity_remaining, &mut party_quantity, price);
                    price_times_quantity.add_assign(quantity_b_traded.clone() * price);
                    quantity_b_traded_total.add_assign(quantity_b_traded);
                    if quantity_remaining == 0u32 {
                        break;
                    }
                }
            }
        }
        if quantity_remaining != 0u32 {
            return Err(TxError::InsufficientLiquidityAvailable);
        }
        let avg_price = price_times_quantity / quantity_b_traded_total;
        nat_to_u64(avg_price)
    }

    fn execute_swap(&mut self, action: Side, quantity: Nat) -> LiquidityReference {
        let mut quantity_remaining = quantity;
        let mut liquidity_reference = LiquidityReference::default();
        match action {
            Side::Buy => {
                for (price, party) in self
                    .asks
                    .iter_mut()
                    .flat_map(|(&price, parties)| parties.into_iter().map(move |p| (price, p)))
                {
                    let original_party_quantity = party.quantity.clone();
                    let _quantity_b_traded =
                        trade(&mut party.quantity.0, &mut quantity_remaining, price);
                    let mut reference = party.clone();
                    reference.quantity = original_party_quantity.sub(reference.quantity);
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
                    let quantity_b_traded =
                        trade(&mut quantity_remaining, &mut party.quantity.0, price);
                    let mut reference = party.clone();
                    reference.quantity = quantity_b_traded.into();
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
