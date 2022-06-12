use candid::{candid_method, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::{has_token_info, is_managed, is_owned};
use enoki_exchange_shared::has_token_info::{AssignedShards, TokenPairInfo};

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

#[query(name = "getManager")]
#[candid_method(query, rename = "getManager")]
fn get_manager() -> Principal {
    is_managed::get_manager()
}

#[update(name = "setManager")]
#[candid_method(update, rename = "setManager")]
fn set_manager(new_manager: Principal) {
    is_managed::set_manager(new_manager).unwrap()
}
