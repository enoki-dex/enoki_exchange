use std::borrow::BorrowMut;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
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
use crate::liquidity::LiquidityReference;

thread_local! {
    static STATE: RefCell<PayoffsState> = RefCell::new(PayoffsState::default());
}

#[derive(Deserialize, CandidType, Clone, Debug, Default)]
struct PayoffsState {
    pending_lp_payoffs: Vec<LpPayoff>
}

#[derive(Deserialize, CandidType, Clone, Debug, Default)]
struct LpPayoff {

}

pub async fn exchange_tokens(order: Order) -> Result<()> {
    todo!()
}

pub async fn exchange_swap(order: ProcessedOrderInput, reference_liquidity: LiquidityReference) -> Result<()> {
    todo!()
}
