use std::borrow::BorrowMut;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::ops::{AddAssign, Div, Mul, Sub, SubAssign};

use candid::{candid_method, CandidType, Deserialize, Nat, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_sharded_users::{get_user_shard, register_user};
use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::has_token_info::{
    get_assigned_shard, get_assigned_shards, nat_to_lots, AssignedShards,
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
    pub fn create_limit_order(
        &mut self,
        caller: Principal,
        side: Side,
        amount: Nat,
        limit_price: Nat,
        allow_taker: bool,
        expiration_time: Option<u64>,
    ) -> Result<u64> {
        self.create_order(
            caller,
            side,
            amount,
            limit_price,
            if allow_taker {
                MakerTaker::MakerOrTaker
            } else {
                MakerTaker::OnlyMaker
            },
            expiration_time,
        )
    }
    pub fn create_market_order(
        &mut self,
        caller: Principal,
        side: Side,
        amount: Nat,
        limit_price: Nat,
    ) -> Result<u64> {
        self.create_order(caller, side, amount, limit_price, MakerTaker::OnlyTaker, None)
    }
    fn get_next_id(&mut self) -> u64 {
        self.last_id += 1;
        self.last_id
    }
    fn create_order(
        &mut self,
        caller: Principal,
        side: Side,
        amount: Nat,
        limit_price: Nat,
        maker_taker: MakerTaker,
        expiration_time: Option<u64>,
    ) -> Result<u64> {
        let id = self.get_next_id();
        let order = OrderInfo {
            broker: ic_cdk::id(),
            user: caller,
            id,
            side,
            maker_taker,
            limit_price: nat_to_lots(&EnokiToken::TokenA, limit_price, true)?,
            quantity: nat_to_lots(&EnokiToken::TokenB, amount, false)?,
            expiration_time,
        };
        if order.quantity == 0 {
            return Err(TxError::IntUnderflow);
        }
        self.new_orders.insert(id, order);
        Ok(id)
    }
}
