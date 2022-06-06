use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Deserialize, Nat, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::is_managed;
use enoki_exchange_shared::is_managed::get_manager;
use enoki_exchange_shared::types::*;

thread_local! {
    static STATE: RefCell<LiquidityState> = RefCell::new(LiquidityState::default());
}

#[derive(Deserialize, CandidType, Clone, Debug, Default)]
struct LiquidityState {
    locked: bool,
    liquidity: HashMap<Principal, LiquidityAmount>,
    pending_add: Vec<(Principal, TokenAmount)>,
    pending_remove: Vec<(Principal, TokenAmount)>,
    pending_add_locked: Vec<(Principal, TokenAmount)>,
    pending_remove_locked: Vec<(Principal, TokenAmount)>,
    add_queue: Vec<(Principal, TokenAmount)>,
    remove_queue: Vec<(Principal, TokenAmount)>,
}

#[query(name = "getLiquidity")]
#[candid_method(query, rename = "getLiquidity")]
fn get_liquidity(user: Principal) -> LiquidityAmount {
    STATE
        .with(|s| s.borrow().liquidity.get(&user).cloned())
        .unwrap_or_default()
}

pub async fn update_liquidity_with_manager() {
    if STATE.with(|s| {
        let s = s.borrow();
        s.locked || (s.pending_add.is_empty() && s.pending_remove.is_empty())
    }) {
        return;
    }
    let (pending_add, pending_remove): (LiquidityAmount, LiquidityAmount) = STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.locked = true;
        s.pending_add_locked = std::mem::take(&mut s.pending_add);
        s.pending_remove_locked = std::mem::take(&mut s.pending_remove);
        (
            s.pending_add_locked.iter().map(|i| i.1.clone()).collect(),
            s.pending_remove_locked
                .iter()
                .map(|i| i.1.clone())
                .collect(),
        )
    });
    let response: Result<(Result<Vec<(Principal, LiquidityAmount)>>,)> = ic_cdk::call(
        get_manager(),
        "updateLiquidity",
        (pending_add, pending_remove),
    )
    .await
    .map_err(|e| e.into());
    let final_result: Result<()> = match response {
        Ok((Ok(result),)) => {
            STATE.with(|s| {
                let mut s = s.borrow_mut();
                s.locked = false;
                let add = std::mem::take(&mut s.pending_add_locked);
                let remove = std::mem::take(&mut s.pending_remove_locked);
                for a in add {
                    s.add_queue.push(a);
                }
                for r in remove {
                    s.remove_queue.push(r);
                }
            });
            distribute_earnings(result).await;
            Ok(())
        }
        Ok((Err(err),)) | Err(err) => {
            // roll back
            STATE.with(|s| {
                let mut s = s.borrow_mut();
                s.locked = false;
                let add = std::mem::take(&mut s.pending_add_locked);
                let remove = std::mem::take(&mut s.pending_remove_locked);
                for a in add {
                    s.pending_add.push(a);
                }
                for r in remove {
                    s.pending_remove.push(r);
                }
            });
            Err(err)
        }
    };
    if let Err(error) = final_result {
        ic_cdk::print(format!(
            "error updating liquidity with manager: {:?}",
            error
        ));
    }
}

async fn distribute_earnings(earnings: Vec<(Principal, LiquidityAmount)>) {
    todo!();
}

#[update(name = "addLiquidity")]
#[candid_method(update, rename = "addLiquidity")]
async fn add_liquidity(from: Principal, to: Principal, value: Nat) -> Result<()> {
    todo!()
}

#[update(name = "removeLiquidity")]
#[candid_method(update, rename = "removeLiquidity")]
async fn remove_liquidity(from: Principal, to: Principal, value: Nat) -> Result<()> {
    todo!()
}
