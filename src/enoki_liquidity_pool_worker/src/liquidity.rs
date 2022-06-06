use std::borrow::BorrowMut;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::ops::{AddAssign, SubAssign};

use candid::{candid_method, CandidType, Deserialize, Nat, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::has_token_info::{get_assigned_shards, AssignedShards};
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
    earnings_pending: Vec<(Principal, TokenAmount)>,
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
    let response: Result<(Result<(LiquidityAmount, LiquidityAmount)>,)> = ic_cdk::call(
        get_manager(),
        "updateLiquidity",
        (pending_add, pending_remove),
    )
    .await
    .map_err(|e| e.into());
    let final_result: Result<Vec<(Principal, TokenAmount)>> = match response {
        Ok((Ok((added, removed)),)) => {
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
                apply_new_liquidity(added, &mut s);
                Ok(calculate_earnings(removed, &mut s))
            })
        }
        Ok((Err(err),)) | Err(err) => {
            // roll back
            STATE.with(|s| {
                let mut s = s.borrow_mut();
                s.locked = false;
                let mut add = std::mem::take(&mut s.pending_add_locked);
                let mut remove = std::mem::take(&mut s.pending_remove_locked);
                s.pending_add.append(&mut add);
                s.pending_remove.append(&mut remove);
            });
            Err(err)
        }
    };
    match final_result {
        Ok(earnings) => {
            distribute_earnings(earnings).await;
        }
        Err(error) => {
            ic_cdk::print(format!(
                "error updating liquidity with manager: {:?}",
                error
            ));
        }
    }
}

fn apply_new_liquidity(mut amount: LiquidityAmount, state: &mut RefMut<LiquidityState>) {
    let mut i = 0;
    while amount.token_a.is_nonzero() || amount.token_b.is_nonzero() {
        if i >= state.add_queue.len() {
            panic!("inconsistent state between pool and worker");
        }
        let item = &mut state.add_queue[i];
        let token = item.1.token.clone();
        let amount_left = amount.get_mut(&token);
        if amount_left.is_nonzero() {
            let diff = amount_left.min(&item.1.amount);
            amount_left.sub_assign(diff.clone());
            item.1.amount.sub_assign(diff.clone());
            let addr = item.0;
            state.liquidity.entry(addr).or_default().get_mut(&token).add_assign(diff);
        }

        i+=1;
    }
    state.add_queue.retain(|(_, amount)| amount.amount.is_nonzero());
}

fn calculate_earnings(mut amount: LiquidityAmount, state: &mut RefMut<LiquidityState>) -> Vec<(Principal, TokenAmount)>{
    let mut amounts_to_distribute: Vec<(Principal, TokenAmount)> = Default::default();
    let mut i = 0;
    while amount.token_a.is_nonzero() || amount.token_b.is_nonzero() {
        if i >= state.remove_queue.len() {
            panic!("inconsistent state between pool and worker");
        }
        let item = &mut state.remove_queue[i];
        let token = item.1.token.clone();
        let amount_left = amount.get_mut(&token);
        if amount_left.is_nonzero() {
            let addr = item.0;
            let amount_in_lp = state.liquidity.entry(addr).or_default().get_mut(&token).clone();
            let item = &mut state.remove_queue[i];
            item.1.amount = item.1.amount.min(&amount_in_lp);
            let diff = amount_left.min(&item.1.amount);
            amount_left.sub_assign(diff.clone());
            item.1.amount.sub_assign(diff.clone());
            state.liquidity.entry(addr).or_default().get_mut(&token).sub_assign(diff.clone());
            amounts_to_distribute.push((addr, TokenAmount { token, amount: diff }))
        }

        i+=1;
    }
    state.remove_queue.retain(|(_, amount)| amount.amount.is_nonzero());
    amounts_to_distribute
}

async fn distribute_earnings(mut earnings: Vec<(Principal, TokenAmount)>) {
    let mut past_pending = STATE.with(|s| std::mem::take(&mut s.borrow_mut().earnings_pending));
    earnings.append(&mut past_pending);
    //TODO: use ShardedPrincipal
    todo!()
}

#[query(name = "getShardsToAddLiquidity")]
#[candid_method(query, rename = "getShardsToAddLiquidity")]
async fn get_shards_to_add_liquidity() -> AssignedShards {
    get_assigned_shards()
}

#[update(name = "addLiquidity")]
#[candid_method(update, rename = "addLiquidity")]
async fn add_liquidity(from: Principal, to: Principal, value: Nat) -> Result<()> {
    assert_eq!(to, ic_cdk::id());
    let token = has_token_info::parse_from()?;
    let amount = TokenAmount {
        token,
        amount: value.into(),
    };
    STATE.with(|s| s.borrow_mut().pending_add.push((from, amount)));
    Ok(())
}

#[update(name = "removeLiquidity")]
#[candid_method(update, rename = "removeLiquidity")]
async fn remove_liquidity(amount: LiquidityAmount) -> Result<()> {
    let from = ic_cdk::caller();

    let result: Result<()> = STATE.with(|s| {
        let mut s = s.borrow_mut();
        let existing = s.liquidity.get(&from).ok_or(TxError::InsufficientFunds)?;
        let amount_a = amount.token_a.min(&existing.token_a);
        let amount_b = amount.token_b.min(&existing.token_b);
        if amount_a.is_nonzero() {
            s.remove_queue.push((from, TokenAmount { token: EnokiToken::TokenA, amount: amount_a }))
        }
        if amount_b.is_nonzero() {
            s.remove_queue.push((from, TokenAmount { token: EnokiToken::TokenB, amount: amount_b }))
        }
        Ok(())
    });
    result?;

    Ok(())
}
