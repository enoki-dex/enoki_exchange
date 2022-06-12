use std::cell::RefCell;

use candid::CandidType;

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

#[allow(unused)]
pub async fn run() {
    if !STATE.with(|s| s.borrow_mut().lock()) {
        return;
    }

    // ic_cdk::api::print("start exchange run");
    let result = do_run().await;
    // ic_cdk::api::print(format!(
    //     "end exchange run {}",
    //     match result {
    //         Ok(_) => "OK".to_string(),
    //         Err(err) => format!("with ERR: {:?}", err),
    //     }
    // ));
    if let Err(error) = result {
        ic_cdk::api::print(format!("ERROR during exchange run: {:?}", error));
    }

    STATE.with(|s| s.borrow_mut().unlock());
}

pub async fn do_run() -> Result<()> {
    if get_broker_ids().is_empty() {
        return Ok(());
    }
    ic_cdk::api::print("started exchange sync");
    let proposed_liquidity_target_for_brokers =
        liquidity::get_updated_liquidity_from_pool().await?;

    ic_cdk::api::print(format!(
        "got liquidity: {:?}",
        proposed_liquidity_target_for_brokers
    ));

    let (new_orders, orders_to_cancel) =
        flat_map_vecs(foreach_broker("retrieveOrders", |_| ()).await?);

    ic_cdk::api::print(format!(
        "got new orders: {:?} / {:?}",
        new_orders, orders_to_cancel
    ));

    let (mut completed_orders, aggregate_bid_ask) = match_orders(new_orders, orders_to_cancel);

    ic_cdk::api::print(format!("completed orders: {:?}", completed_orders));

    STATE.with(|s| {
        s.borrow_mut()
            .aggregate_bid_ask
            .change_to_next(&aggregate_bid_ask)
    });

    ic_cdk::api::print("submitting orders to brokers...");

    let changes_in_liquidity_by_broker = foreach_broker_map(
        "submitCompletedOrders",
        |id| {
            (
                completed_orders.remove(&id).unwrap_or_default(),
                aggregate_bid_ask.clone(),
                proposed_liquidity_target_for_brokers.clone(),
            )
        },
        |res: (ResponseAboutLiquidityChanges, )| res.0,
    )
        .await?;

    ic_cdk::api::print("updating changes in liquidity...");

    update_committed_broker_liquidity(changes_in_liquidity_by_broker).await?;

    ic_cdk::api::print("end exchange sync");

    Ok(())
}

pub fn export_stable_storage() -> RunningState {
    STATE.with(|s| s.take())
}

pub fn import_stable_storage(data: RunningState) {
    STATE.with(|s| s.replace(data));
}
