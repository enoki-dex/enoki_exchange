use candid::{candid_method, CandidType, Deserialize, Principal, Nat};
use ic_cdk_macros::*;
use crate::liquidity::update_liquidity_with_manager;

#[heartbeat]
fn tick() {
    ic_cdk::spawn(update_liquidity_with_manager())
}
