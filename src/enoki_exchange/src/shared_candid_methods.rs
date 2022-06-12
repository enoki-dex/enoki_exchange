use candid::{candid_method, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::{has_token_info, has_trading_fees, is_owned};
use enoki_exchange_shared::has_token_info::{AssignedShards, TokenPairInfo};
use enoki_exchange_shared::has_trading_fees::TradingFees;

#[query(name = "getOwner")]
#[candid_method(query, rename = "getOwner")]
fn get_owner() -> Principal {
    is_owned::get_owner()
}

#[update(name = "setOwner")]
#[candid_method(update, rename = "setOwner")]
fn set_owner(new_owner: Principal) {
    is_owned::set_owner(new_owner).unwrap()
}

#[query(name = "getTokenInfo")]
#[candid_method(query, rename = "getTokenInfo")]
fn get_token_info() -> TokenPairInfo {
    has_token_info::get_token_info()
}

#[query(name = "getAssignedShards")]
#[candid_method(query, rename = "getAssignedShards")]
fn get_assigned_shards() -> AssignedShards {
    has_token_info::get_assigned_shards()
}

#[query(name = "getAssignedShardA")]
#[candid_method(query, rename = "getAssignedShardA")]
fn get_assigned_shard_a() -> Principal {
    has_token_info::get_assigned_shard_a()
}

#[query(name = "getAssignedShardB")]
#[candid_method(query, rename = "getAssignedShardB")]
fn get_assigned_shard_b() -> Principal {
    has_token_info::get_assigned_shard_b()
}

#[query(name = "getTradingFees")]
#[candid_method(query, rename = "getTradingFees")]
fn get_trading_fees() -> TradingFees {
    has_trading_fees::get_trading_fees()
}
