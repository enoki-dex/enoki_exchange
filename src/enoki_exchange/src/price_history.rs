use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::VecDeque;

use candid::{candid_method, CandidType};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info::price_in_b_u64_to_float;

const MAX_HISTORY_SIZE: usize = 3600;

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct PriceHistory {
    pub last_prices_by_timestamp: VecDeque<LastPrice>,
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct LastPrice {
    pub price: u64,
    pub time: u64,
    pub price_was_lifted: bool,
}

#[derive(candid::Deserialize, CandidType, Clone, Debug, Default)]
pub struct LastPricePoint {
    pub price: f64,
    pub time: u64,
    pub price_was_lifted: bool,
}

thread_local! {
    static STATE: RefCell<PriceHistory> = RefCell::new(PriceHistory::default());
}

pub fn save_last_price(last_price: LastPrice) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.last_prices_by_timestamp.push_back(last_price);
        if s.last_prices_by_timestamp.len() > MAX_HISTORY_SIZE {
            s.last_prices_by_timestamp.pop_front();
        }
    })
}

pub fn save_last_price_value(last_price: u64) {
    let price_was_lifted = STATE.with(|s| {
        if let Some(previous) = s.borrow().last_prices_by_timestamp.iter().last() {
            match last_price.cmp(&previous.price) {
                Ordering::Less => false,
                Ordering::Equal => previous.price_was_lifted,
                Ordering::Greater => true,
            }
        } else {
            true
        }
    });
    save_last_price(LastPrice {
        price: last_price,
        time: ic_cdk::api::time(),
        price_was_lifted,
    })
}

pub fn get_last_price_time() -> u64 {
    STATE
        .with(|s| {
            s.borrow()
                .last_prices_by_timestamp
                .iter()
                .last()
                .map(|p| p.time)
        })
        .unwrap_or_default()
}

#[query(name = "getPriceHistory")]
#[candid_method(query, rename = "getPriceHistory")]
fn get_price_history() -> Vec<LastPricePoint> {
    STATE.with(|s| {
        s.borrow()
            .last_prices_by_timestamp
            .iter()
            .map(|last| LastPricePoint {
                price: price_in_b_u64_to_float(last.price),
                time: last.time,
                price_was_lifted: last.price_was_lifted,
            })
            .collect()
    })
}

pub fn export_stable_storage() -> PriceHistory {
    STATE.with(|s| s.take())
}

pub fn import_stable_storage(data: PriceHistory) {
    STATE.with(|s| s.replace(data));
}
