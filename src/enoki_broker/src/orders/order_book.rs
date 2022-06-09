use std::borrow::BorrowMut;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::ops::{AddAssign, Div, Mul, Sub, SubAssign};

use candid::{candid_method, CandidType, Deserialize, Nat, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_sharded_users::{get_user_shard, register_user};
use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::has_token_info::{
    get_assigned_shard, get_assigned_shards, price_in_b_float_to_u64,
    quantity_token_b_nat_to_trade_units, AssignedShards,
};
use enoki_exchange_shared::types::*;

#[derive(Deserialize, CandidType, Clone, Debug, Default)]
pub struct OrderBook {
    last_id: u64,
    new_orders: HashMap<u64, OrderInfo>,
    pending_orders: HashMap<u64, OrderInfo>,
    orders_to_cancel: HashMap<u64, OrderInfo>,
    pending_orders_to_cancel: HashMap<u64, OrderInfo>,
}

impl OrderBook {
    pub fn create_limit_order(&mut self, input: ProcessedOrderInput) -> Result<u64> {
        self.create_order(input)
    }
    pub fn create_market_order(&mut self, input: ProcessedOrderInput) -> Result<u64> {
        self.create_order(input)
    }
    fn get_next_id(&mut self) -> u64 {
        self.last_id += 1;
        self.last_id
    }
    fn create_order(&mut self, input: ProcessedOrderInput) -> Result<u64> {
        let id = self.get_next_id();
        let order = OrderInfo {
            broker: ic_cdk::id(),
            user: input.user,
            id,
            side: input.side,
            maker_taker: input.maker_taker,
            limit_price: price_in_b_float_to_u64(input.limit_price_in_b)?,
            quantity: input.quantity.into(),
            expiration_time: input.expiration_time,
        };
        if !order.quantity.is_nonzero() {
            return Err(TxError::IntUnderflow);
        }
        self.new_orders.insert(id, order);
        Ok(id)
    }
}
