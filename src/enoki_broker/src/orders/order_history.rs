use std::borrow::BorrowMut;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::ops::{AddAssign, Div, Mul, Sub, SubAssign};

use candid::{CandidType, Deserialize, Nat, Principal};

use enoki_exchange_shared::types::*;

#[derive(Deserialize, CandidType, Clone, Debug, Default)]
pub struct OrderHistory {
    current_orders: HashMap<Principal, Vec<u64>>,
    past_orders: HashMap<Principal, HashMap<u64, Order>>,
}

impl OrderHistory {

}
