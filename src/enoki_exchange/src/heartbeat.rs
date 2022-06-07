use std::borrow::BorrowMut;
use std::cell::RefCell;

use candid::{candid_method, CandidType, Deserialize, Nat, Principal};
use ic_cdk_macros::*;
use crate::synchronize::run;

#[heartbeat]
fn tick() {
    ic_cdk::spawn(run())
}