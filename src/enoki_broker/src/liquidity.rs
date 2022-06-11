use std::cell::{RefCell};
use std::collections::{BTreeMap, HashMap};
use std::ops::{AddAssign, Sub, SubAssign};

use candid::{CandidType, Nat};
use num_traits::cast::ToPrimitive;

use enoki_exchange_shared::has_token_info::{
    quantity_a_to_b, quantity_b_to_a, QuantityTranslator,
};
use enoki_exchange_shared::has_trading_fees::{get_swap_fee, get_swap_market_maker_reward};
use enoki_exchange_shared::liquidity::{
    RequestForNewLiquidityTarget, ResponseAboutLiquidityChanges,
};
use enoki_exchange_shared::types::*;
use enoki_exchange_shared::utils::{nat_to_u64, nat_x_float};

use crate::payoffs;

thread_local! {
    static STATE: RefCell<LiquidityState> = RefCell::new(LiquidityState::default());
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct LiquidityState {
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

pub async fn swap(mut order: ProcessedOrderInput) {
    let swap_fee = get_swap_fee();
    let original_quantity = order.quantity.clone();
    order.quantity = nat_x_float(order.quantity, 1.0 - swap_fee).unwrap();
    let mut lp_credit = original_quantity - order.quantity.clone();
    let market_maker_percentage = get_swap_market_maker_reward();
    let market_maker_reward = nat_x_float(lp_credit.clone(), market_maker_percentage).unwrap();
    lp_credit -= market_maker_reward.clone();

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
    let (token_supplier, token_user) = match &order.side {
        Side::Buy => (EnokiToken::TokenA, EnokiToken::TokenB),
        Side::Sell => (EnokiToken::TokenB, EnokiToken::TokenA),
    };
    let traded = STATE.with(|s| {
        let mut s = s.borrow_mut();
        let mut traded = LiquidityTrades::default();
        let quantity_supplier: StableNat = swap
            .prices
            .iter()
            .flat_map(|(_p, val)| val.iter().map(|info| info.quantity.clone()))
            .sum();
        let quantity_user = order.quantity.clone();
        traded
            .increased
            .get_mut(&token_user)
            .add_assign(StableNat(quantity_user + lp_credit));
        traded
            .decreased
            .get_mut(&token_supplier)
            .add_assign(quantity_supplier);
        s.available_liquidity.sub_assign(traded.decreased.clone());
        traded
    });
    if let Err(error) = payoffs::send_swap_tokens(
        order.user,
        &token_supplier,
        traded.decreased.get(&token_supplier).0.clone(),
    )
    .await
    {
        STATE.with(|s| {
            s.borrow_mut()
                .available_liquidity
                .add_assign(traded.decreased)
        });
        panic!("{:?}", error);
    }
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.available_liquidity.add_assign(traded.increased.clone());
        s.liquidity_traded.add_assign(traded);
    });
    pay_rewards_to_market_makers(
        market_maker_reward,
        match &order.side {
            Side::Buy => &EnokiToken::TokenB,
            Side::Sell => &EnokiToken::TokenA,
        },
        swap,
    );
}

fn pay_rewards_to_market_makers(
    reward: Nat,
    reward_token: &EnokiToken,
    reference: LiquidityReference,
) {
    let amount_by_user = reference.get_map_of_complement_token_by_broker(reward_token);
    let total = amount_by_user
        .values()
        .fold(Nat::default(), |sum, next| sum + next.clone());
    for (BrokerAndUser { broker, user }, amount_provided) in amount_by_user {
        let user_reward = nat_x_float(
            reward.clone(),
            amount_provided.0.to_f64().unwrap() / total.0.to_f64().unwrap(),
        )
        .unwrap();
        payoffs::add_reward(broker, user, reward_token, user_reward);
    }
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

impl LiquidityReference {
    pub fn get_map_of_complement_token_by_broker(
        &self,
        complement_token: &EnokiToken,
    ) -> HashMap<BrokerAndUser, Nat> {
        self.prices
            .iter()
            .flat_map(|(&price, parties)| {
                parties.into_iter().map(move |party| {
                    (
                        BrokerAndUser {
                            broker: party.broker,
                            user: party.user,
                        },
                        match complement_token {
                            EnokiToken::TokenA => {
                                quantity_b_to_a(party.quantity.0.clone(), price).unwrap()
                            }
                            EnokiToken::TokenB => {
                                quantity_a_to_b(party.quantity.0.clone(), price).unwrap()
                            }
                        },
                    )
                })
            })
            .fold(HashMap::new(), |mut map, next| {
                map.entry(next.0).or_default().add_assign(next.1);
                map
            })
    }
}

pub fn export_stable_storage() -> LiquidityState {
    STATE.with(|s| s.take())
}

pub fn import_stable_storage(data: LiquidityState) {
    STATE.with(|s| s.replace(data));
}
