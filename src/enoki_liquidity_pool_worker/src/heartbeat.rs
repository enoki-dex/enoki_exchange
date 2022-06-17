use std::cell::RefCell;

use candid::candid_method;
use ic_cdk_macros::*;

use crate::liquidity::update_liquidity_with_manager;

// #[heartbeat]
// fn tick() {
//     ic_cdk::spawn(update_liquidity_with_manager())
// }

thread_local! {
    static STATE: RefCell<AntiSpam> = RefCell::new(Default::default());
}

#[derive(Default)]
struct AntiSpam {
    last_update: u64,
    locked: bool,
}

impl AntiSpam {
    const UPDATE_INTERVAL: u64 = 1 * 1_000_000_000;
    pub fn try_lock(&mut self) -> bool {
        let now = ic_cdk::api::time();
        if self.last_update < now - Self::UPDATE_INTERVAL {
            self.locked = true;
            self.last_update = now;
            true
        } else {
            false
        }
    }
    pub fn unlock(&mut self) {
        self.locked = false;
    }
}

// without too many users, this is probably much cheaper
#[update(name = "triggerHeartbeat")]
#[candid_method(update, rename = "triggerHeartbeat")]
async fn trigger_heartbeat() -> Option<u64> {
    if let Some(too_soon) = STATE.with(|s| {
        let mut s = s.borrow_mut();
        if s.try_lock() {
            None
        } else {
            Some(s.last_update)
        }
    }) {
        return Some(too_soon);
    }
    update_liquidity_with_manager().await;
    STATE.with(|s| s.borrow_mut().unlock());
    None
}
