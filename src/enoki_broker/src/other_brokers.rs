use std::borrow::BorrowMut;
use std::cell::{RefCell, RefMut};
use std::collections::{HashMap, HashSet};
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
use enoki_exchange_shared::is_managed::assert_is_manager;
use enoki_exchange_shared::types::*;

thread_local! {
    static STATE: RefCell<BrokersState> = RefCell::new(BrokersState::default());
}

pub fn assert_is_broker(principal: Principal) -> Result<()> {
    if STATE.with(|s| s.borrow().other_brokers.contains(&principal)) {
        Ok(())
    } else {
        Err(TxError::Unauthorized)
    }
}

#[update(name = "addBroker")]
#[candid_method(update, rename = "addBroker")]
fn add_broker(principal: Principal) {
    assert_is_manager().unwrap();
    STATE.with(|s| s.borrow_mut().other_brokers.insert(principal));
}

pub fn init_brokers(brokers: Vec<Principal>) {
    STATE.with(|s| s.borrow_mut().other_brokers.extend(brokers));
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct BrokersState {
    other_brokers: HashSet<Principal>,
}

pub fn export_stable_storage() -> BrokersState {
    STATE.with(|s| s.take())
}

pub fn import_stable_storage(data: BrokersState) {
    STATE.with(|s| s.replace(data));
}
