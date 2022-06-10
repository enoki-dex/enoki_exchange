use std::collections::{BTreeMap, HashMap};

use candid::{candid_method, CandidType, Nat, Principal};

use enoki_exchange_shared::types::*;

use crate::orders::bid_ask::BidAsk;
use crate::orders::matching::OrderMatching;

#[derive(serde::Deserialize, serde::Serialize, CandidType, Clone, Debug, Default)]
pub struct OrderMatcher {
    bids: BidAsk,
    asks: BidAsk,
}

impl OrderMatcher {
    fn open_orders(&mut self, side: &Side) -> &mut BidAsk {
        match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        }
    }
    fn add_maker_only(&mut self, mut order: Order) -> Option<Order> {
        match order.info.side {
            Side::Buy => {
                if let Some(ask) = self.asks.get_lowest_price() {
                    if ask <= order.info.limit_price {
                        order.state.status = OrderStatus::InvalidPrice;
                        return Some(order);
                    }
                }
                self.bids.insert(order);
            }
            Side::Sell => {
                if let Some(bid) = self.bids.get_highest_price() {
                    if bid >= order.info.limit_price {
                        order.state.status = OrderStatus::InvalidPrice;
                        return Some(order);
                    }
                }
                self.asks.insert(order);
            }
        }
        None
    }
    pub fn match_orders(
        &mut self,
        new_orders: Vec<OrderInfo>,
        orders_to_cancel: Vec<OrderInfo>,
    ) -> (HashMap<Principal, Vec<Order>>, AggregateBidAsk) {
        let mut completed_orders: CompletedOrders = Default::default();

        let mut only_makers: Vec<Order> = Vec::new();
        let mut maker_taker: Vec<Order> = Vec::new();
        let mut only_takers: Vec<Order> = Vec::new();
        for order in new_orders {
            match order.maker_taker {
                MakerTaker::OnlyMaker => only_makers.push(order.into()),
                MakerTaker::OnlyTaker => only_takers.push(order.into()),
                MakerTaker::MakerOrTaker => maker_taker.push(order.into()),
            }
        }

        // try to insert maker-only orders
        for maker in only_makers {
            if let Some(invalid) = self.add_maker_only(maker) {
                completed_orders.insert(invalid);
            }
        }

        // execute or insert maker-taker orders
        for mut order in maker_taker {
            match order.info.side {
                Side::Buy => {
                    self.asks.try_match_with_asks(&mut order);
                    if order.is_complete() {
                        completed_orders.insert(order);
                    } else {
                        self.bids.insert(order);
                    }
                }
                Side::Sell => {
                    self.bids.try_match_with_bids(&mut order);
                    if order.is_complete() {
                        completed_orders.insert(order);
                    } else {
                        self.asks.insert(order);
                    }
                }
            }
        }

        // execute or cancel taker-only orders
        for mut order in only_takers {
            match order.info.side {
                Side::Buy => {
                    self.asks.try_match_with_asks(&mut order);
                    if !order.is_complete() {
                        order.state.status = OrderStatus::InsufficientLiquidity;
                    }
                    completed_orders.insert(order);
                }
                Side::Sell => {
                    self.bids.try_match_with_bids(&mut order);
                    if !order.is_complete() {
                        order.state.status = OrderStatus::InsufficientLiquidity;
                    }
                    completed_orders.insert(order);
                }
            }
        }

        // try to cancel existing orders.
        // Do this step last to help prevent a potential "arbitrage" attack where a market maker
        // submits a large order with a price that crosses the bid/ask, waits (2 consensus rounds)
        // for the LP bid/ask curve to change, swaps, and then cancels the original order.
        // Other than this measure, further measures are probably not necessary because other market
        // makers will probably intervene and profit (higher risk of loss for attacker).
        // However, another measure could be to keep track of the swap trades and prices, and
        // match those with trades to be cancelled.
        for to_cancel in orders_to_cancel {
            if let Some(cancelled) = self.open_orders(&to_cancel.side).try_cancel(&to_cancel) {
                completed_orders.insert(cancelled);
            }
        }

        // this means orders are actually cancelled a few seconds after they expire
        self.bids.cancel_expired();
        self.asks.cancel_expired();

        for completed in self
            .asks
            .take_completed()
            .into_iter()
            .chain(self.bids.take_completed())
        {
            completed_orders.insert(completed);
        }

        (
            completed_orders.0,
            AggregateBidAsk {
                bids: self.bids.get_counterparty_info(),
                asks: self.asks.get_counterparty_info(),
            },
        )
    }
}

#[derive(Default)]
struct CompletedOrders(HashMap<Principal, Vec<Order>>);

impl CompletedOrders {
    pub fn insert(&mut self, order: Order) {
        self.0.entry(order.info.broker).or_default().push(order);
    }
}
