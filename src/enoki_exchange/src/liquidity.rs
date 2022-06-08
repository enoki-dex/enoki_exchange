use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{AddAssign, DivAssign, SubAssign};

use candid::{candid_method, CandidType, Nat, Principal};
use futures::AsyncReadExt;
use ic_cdk_macros::*;

use enoki_exchange_shared::liquidity::{RequestForLiquidityChanges, ResponseAboutLiquidityChanges};
use enoki_exchange_shared::types::*;
use enoki_exchange_shared::utils::map_assign;

thread_local! {
    static STATE: RefCell<LiquidityState> = RefCell::new(LiquidityState::default());
}

#[derive(serde::Deserialize, serde::Serialize, CandidType, Clone, Debug)]
pub struct LiquidityState {
    pool_address: Principal,
    broker_liquidity: HashMap<Principal, HashMap<Principal, LiquidityAmount>>,
    excess_liquidity: HashMap<Principal, LiquidityAmount>,
    proposed_changes: ProposedLiquidityChangesByWorker,
}

impl Default for LiquidityState {
    fn default() -> Self {
        Self {
            pool_address: Principal::anonymous(),
            broker_liquidity: Default::default(),
            excess_liquidity: Default::default(),
            proposed_changes: Default::default(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, CandidType, Clone, Debug, Default)]
pub struct ProposedLiquidityChangesByWorker {
    to_add: HashMap<Principal, LiquidityAmount>,
    to_remove: HashMap<Principal, LiquidityAmount>,
}

pub fn init_pool(pool: Principal) {
    STATE.with(|s| s.borrow_mut().pool_address = pool);
}

pub fn add_broker(broker: Principal) {
    STATE.with(|s| {
        s.borrow_mut()
            .broker_liquidity
            .insert(broker, Default::default())
    });
}

pub fn get_pool_contract() -> Principal {
    STATE.with(|s| s.borrow().pool_address)
}

pub async fn get_updated_liquidity_from_pool(
) -> Result<HashMap<Principal, RequestForLiquidityChanges>> {
    let result: Result<(
        HashMap<Principal, LiquidityAmount>,
        HashMap<Principal, LiquidityAmount>,
    )> = ic_cdk::call(get_pool_contract(), "getUpdatedLiquidity", ())
        .await
        .map_err(|e| e.into());
    let (to_add, to_remove) = result?;
    let proposed_changes_by_broker =
        create_requests_for_broker_liquidity_changes(&to_add, &to_remove);
    STATE.with(|s| {
        s.borrow_mut().proposed_changes = ProposedLiquidityChangesByWorker { to_add, to_remove }
    });
    Ok(proposed_changes_by_broker)
}

fn create_requests_for_broker_liquidity_changes(
    proposed_added_liquidity: &HashMap<Principal, LiquidityAmount>,
    proposed_remove_liquidity: &HashMap<Principal, LiquidityAmount>,
) -> HashMap<Principal, RequestForLiquidityChanges> {
    let current_liquidity = STATE.with(|s| s.borrow().broker_liquidity.clone());
    let broker_count = current_liquidity.len();

    //TODO: maybe a flat map with respect to lp workers gives a better target total liquidity,
    // but this is simpler for now, especially with only 1 lp worker
    let mut total_liquidity_target: HashMap<Principal, LiquidityAmount> = current_liquidity
        .into_iter()
        .map(|(_broker, current_amount)| current_amount)
        .fold(HashMap::new(), |mut sum, next| {
            map_assign(&mut sum, next, |s, n| s.add_assign(n));
            sum
        });
    map_assign(
        &mut total_liquidity_target,
        proposed_added_liquidity.clone(),
        |s, n| s.add_assign(n),
    );
    map_assign(
        &mut total_liquidity_target,
        STATE.with(|s| s.borrow().excess_liquidity.clone()),
        |s, n| s.add_assign(n),
    );
    map_assign(
        &mut total_liquidity_target,
        proposed_remove_liquidity.clone(),
        |s, n| s.sub_assign_or_zero(n),
    );

    for (_, liq) in total_liquidity_target.iter_mut() {
        liq.div_assign_int(broker_count);
    }
    let target_liquidity_per_broker = total_liquidity_target;

    target_liquidity_per_broker
        .into_iter()
        .map(|(broker, target)| {
            
        });
    todo!()
}

pub async fn update_committed_broker_liquidity(
    response: HashMap<Principal, ResponseAboutLiquidityChanges>,
) -> Result<()> {
    apply_changes(&response);
    let proposed_by_lp = STATE.with(|s| std::mem::take(&mut s.borrow_mut().proposed_changes));
    let (mut added, mut removed, traded): (
        HashMap<Principal, LiquidityAmount>,
        HashMap<Principal, LiquidityAmount>,
        HashMap<Principal, LiquidityTrades>,
    ) = response.into_iter().fold(
        (Default::default(), Default::default(), Default::default()),
        |(mut added, mut removed, mut traded): (
            HashMap<Principal, LiquidityAmount>,
            HashMap<Principal, LiquidityAmount>,
            HashMap<Principal, LiquidityTrades>,
        ),
         (_, changes)| {
            for (id, a) in changes.added.into_iter() {
                added.entry(id).or_default().add_assign(a);
            }
            for (id, r) in changes.removed.into_iter() {
                removed.entry(id).or_default().add_assign(r);
            }
            for (id, t) in changes.traded.into_iter() {
                traded.entry(id).or_default().add_assign(t);
            }
            (added, removed, traded)
        },
    );
    let mut excess_added: HashMap<Principal, LiquidityAmount> = added
        .iter_mut()
        .map(|(worker, added_amount)| {
            if let Some(proposed_add) = proposed_by_lp.to_add.get(worker) {
                let diff = added_amount.diff_or_zero(proposed_add);
                added_amount.sub_assign(diff.clone());
                (*worker, diff)
            } else {
                (*worker, LiquidityAmount::default())
            }
        })
        .collect();
    let mut excess_removed: HashMap<Principal, LiquidityAmount> = removed
        .iter_mut()
        .map(|(worker, removed_amount)| {
            if let Some(proposed_remove) = proposed_by_lp.to_remove.get(worker) {
                let diff = removed_amount.diff_or_zero(proposed_remove);
                removed_amount.sub_assign(diff.clone());
                (*worker, diff)
            } else {
                (*worker, LiquidityAmount::default())
            }
        })
        .collect();
    STATE.with(|s| {
        for (worker, amount) in s.borrow_mut().excess_liquidity.iter_mut() {
            amount.add_assign(excess_removed.remove(worker).unwrap_or_default());
            amount.sub_assign(excess_added.remove(worker).unwrap_or_default());
        }
    });

    let result: Result<()> = ic_cdk::call(
        get_pool_contract(),
        "resolveLiquidity",
        (added, removed, traded),
    )
    .await
    .map_err(|e| e.into());
    result
}

fn apply_changes(changes: &HashMap<Principal, ResponseAboutLiquidityChanges>) {
    STATE.with(|s| {
        for (broker_id, liquidity) in s.borrow_mut().broker_liquidity.iter_mut() {
            if let Some(changes) = changes.get(broker_id) {
                for (&worker_id, traded) in &changes.traded {
                    liquidity
                        .entry(worker_id)
                        .or_default()
                        .add_assign(traded.increased.clone());
                    liquidity
                        .entry(worker_id)
                        .or_default()
                        .sub_assign(traded.decreased.clone());
                }
                for (&worker_id, added) in &changes.added {
                    liquidity
                        .entry(worker_id)
                        .or_default()
                        .add_assign(added.clone());
                }
                for (&worker_id, removed) in &changes.removed {
                    liquidity
                        .entry(worker_id)
                        .or_default()
                        .sub_assign(removed.clone());
                }
            }
        }
    })
}

pub fn export_stable_storage() -> (LiquidityState,) {
    let data = STATE.with(|s| s.take());
    (data,)
}

pub fn import_stable_storage(data: LiquidityState) {
    STATE.with(|s| s.replace(data));
}
