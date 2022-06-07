use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Nat, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::types::*;
use enoki_exchange_shared::utils::flat_map_vecs;

use crate::brokers::foreach_broker;
use crate::orders::{match_orders, take_cancelled_orders, Order};

thread_local! {
    static STATE: RefCell<RunningState> = RefCell::new(RunningState::default());
}

#[derive(serde::Deserialize, serde::Serialize, CandidType, Clone, Debug, Default)]
pub struct RunningState {
    locked: bool,
}

impl RunningState {
    pub fn lock(&mut self) -> bool {
        if self.locked {
            false
        } else {
            self.locked = true;
            true
        }
    }
    pub fn unlock(&mut self) {
        self.locked = false;
    }
}

pub async fn run() {
    if !STATE.with(|s| s.borrow_mut().lock()) {
        return;
    }

    let result = do_run().await;

    if let Err(error) = result {
        ic_cdk::api::print(format!("error with run: {:?}", error));
    }

    STATE.with(|s| s.borrow_mut().unlock());
}

async fn do_run() -> Result<()> {
    let cancelled_orders = take_cancelled_orders();

    let (new_orders, orders_to_cancel) = flat_map_vecs(
        foreach_broker::<(HashMap<Principal, Vec<Order>>,), (Vec<Order>, Vec<u64>)>(
            "retrieve_orders",
            (cancelled_orders,),
        )
        .await?,
    );

    match_orders(new_orders, orders_to_cancel);

    Ok(())
}

pub fn export_stable_storage() -> (RunningState,) {
    let data = STATE.with(|s| s.take());
    (data,)
}

pub fn import_stable_storage(data: RunningState) {
    STATE.with(|s| s.replace(data));
}
