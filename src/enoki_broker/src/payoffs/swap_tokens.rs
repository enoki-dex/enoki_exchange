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
use crate::token_liquidity_params::{get_lp_worker_assigned_shard, get_lp_worker_location};

pub async fn send_swap_tokens(
    user: Principal,
    token: &EnokiToken,
    amount_to_send: Nat,
) -> Result<()> {
    let lp_location = get_lp_worker_location();
    let lp_shard = get_lp_worker_assigned_shard(token);
    let user_shard = get_user_shard(user, get_token_address(token))?;
    let result: Result<()> = ic_cdk::call(
        lp_shard,
        "shardSpend",
        (lp_location, user_shard, user, amount_to_send),
    )
    .await
    .map_err(|e| e.into());
    result
}
