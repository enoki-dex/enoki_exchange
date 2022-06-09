use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Nat, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::liquidity::ResponseAboutLiquidityChanges;
use enoki_exchange_shared::types::*;
use enoki_exchange_shared::utils::flat_map_vecs;

use crate::brokers::{foreach_broker, foreach_broker_map, get_broker_ids};
use crate::liquidity;
use crate::liquidity::update_committed_broker_liquidity;
use crate::orders::match_orders;

thread_local! {
    static STATE: RefCell<RunningState> = RefCell::new(RunningState::default());
}

#[derive(serde::Deserialize, serde::Serialize, CandidType, Clone, Debug, Default)]
pub struct RunningState {
    locked: bool,
    aggregate_bid_ask: AggregateBidAsk,
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
    let proposed_liquidity_target_for_brokers =
        liquidity::get_updated_liquidity_from_pool().await?;

    let (new_orders, orders_to_cancel) =
        flat_map_vecs(foreach_broker("retrieveOrders", |_| ()).await?);

    let (mut completed_orders, aggregate_bid_ask) = match_orders(new_orders, orders_to_cancel);

    STATE.with(|s| {
        s.borrow_mut()
            .aggregate_bid_ask
            .change_to_next(&aggregate_bid_ask)
    });

    let changes_in_liquidity_by_broker = foreach_broker_map(
        "receiveCompletedOrders",
        |id| {
            (
                completed_orders.remove(&id).unwrap_or_default(),
                aggregate_bid_ask.clone(),
                proposed_liquidity_target_for_brokers.clone(),
            )
        },
        |res: (ResponseAboutLiquidityChanges,)| res.0,
    )
    .await?;

    update_committed_broker_liquidity(changes_in_liquidity_by_broker).await?;

    Ok(())
}

pub fn export_stable_storage() -> (RunningState,) {
    let data = STATE.with(|s| s.take());
    (data,)
}

pub fn import_stable_storage(data: RunningState) {
    STATE.with(|s| s.replace(data));
}
