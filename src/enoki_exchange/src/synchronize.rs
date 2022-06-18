use std::cell::RefCell;

use candid::{candid_method, CandidType, Nat};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::has_token_info::quantity_b_to_a;
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

    let result = do_run().await;
    if let Err(error) = result {
        ic_cdk::api::print(format!("ERROR during exchange run: {:?}", error));
    }

    STATE.with(|s| s.borrow_mut().unlock());
}

pub async fn do_run() -> Result<()> {
    if get_broker_ids().is_empty() {
        return Ok(());
    }
    ic_cdk::println!("[exchange] started exchange sync");
    let proposed_liquidity_target_for_brokers =
        liquidity::get_updated_liquidity_from_pool().await?;

    ic_cdk::api::print(format!(
        "[exchange] got liquidity: {:?}",
        proposed_liquidity_target_for_brokers
    ));

    let (new_orders, orders_to_cancel) =
        flat_map_vecs(foreach_broker("retrieveOrders", |_| ()).await?);

    ic_cdk::println!(
        "[exchange] got {} new orders and {} to cancel",
        new_orders.len(),
        orders_to_cancel.len()
    );

    let (mut completed_orders, aggregate_bid_ask) = match_orders(new_orders, orders_to_cancel);

    ic_cdk::println!("[exchange] completed orders: {:?}", completed_orders);

    STATE.with(|s| {
        s.borrow_mut()
            .aggregate_bid_ask
            .change_to_next(&aggregate_bid_ask)
    });

    ic_cdk::println!("[exchange] submitting orders to brokers...");

    let changes_in_liquidity_by_broker = foreach_broker_map(
        "submitCompletedOrders",
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

    ic_cdk::println!("[exchange] updating changes in liquidity...");

    update_committed_broker_liquidity(changes_in_liquidity_by_broker).await?;

    ic_cdk::println!("[exchange] end exchange sync");

    Ok(())
}

#[query(name = "getBidAskCurve")]
#[candid_method(query, rename = "getBidAskCurve")]
pub fn get_bid_ask_curve() -> BidAskCurve {
    let num_decimals = has_token_info::get_number_of_price_decimals();
    let bid_ask = STATE.with(|s| s.borrow().aggregate_bid_ask.clone());
    fn sum(info: Vec<CounterpartyInfo>) -> Nat {
        info.into_iter()
            .fold(Nat::from(0u32), |sum, next| sum + next.quantity.to_nat())
    }
    BidAskCurve {
        num_decimals,
        bids: bid_ask
            .bids
            .into_iter()
            .map(|(price, bids)| (price, quantity_b_to_a(sum(bids), price).unwrap()))
            .collect(),
        asks: bid_ask
            .asks
            .into_iter()
            .map(|(price, asks)| (price, sum(asks)))
            .collect(),
    }
}

pub fn export_stable_storage() -> RunningState {
    STATE.with(|s| s.take())
}

pub fn import_stable_storage(data: RunningState) {
    STATE.with(|s| s.replace(data));
}
