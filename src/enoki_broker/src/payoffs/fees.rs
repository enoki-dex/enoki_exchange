use std::borrow::BorrowMut;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::convert::TryInto;
use std::ops::{AddAssign, Div, Mul, Sub, SubAssign};

use candid::parser::token::Token;
use candid::{candid_method, CandidType, Deserialize, Nat, Principal};
use futures::FutureExt;
use ic_cdk_macros::*;

use enoki_exchange_shared::has_sharded_users::{get_user_shard, register_user};
use enoki_exchange_shared::has_token_info::{
    get_assigned_shard, get_assigned_shards, get_token_address, price_in_b_float_to_u64,
    AssignedShards,
};
use enoki_exchange_shared::has_trading_fees::{get_deposit_fee, TradingFees};
use enoki_exchange_shared::interfaces::enoki_wrapped_token::ShardedTransferNotification;
use enoki_exchange_shared::is_managed;
use enoki_exchange_shared::is_managed::{assert_is_manager, get_manager};
use enoki_exchange_shared::liquidity::liquidity_pool::LiquidityPool;
use enoki_exchange_shared::liquidity::{
    RequestForNewLiquidityTarget, ResponseAboutLiquidityChanges,
};
use enoki_exchange_shared::types::*;
use enoki_exchange_shared::{has_token_info, has_trading_fees};

thread_local! {
    static STATE: RefCell<AccruedFees> = RefCell::new(AccruedFees::default());
}

#[derive(Deserialize, CandidType, Clone, Debug, Default)]
struct AccruedFees {
    deposit_fees: LiquidityAmount,
}

pub fn charge_deposit_fee(token: &EnokiToken, deposit_amount: Nat) -> Nat {
    let fee = get_deposit_fee(token);
    let remaining = deposit_amount - fee.clone();
    STATE.with(|s| {
        s.borrow_mut()
            .deposit_fees
            .get_mut(&token)
            .add_assign(fee.into())
    });
    remaining
}

#[update(name = "setFees")]
#[candid_method(update, rename = "setFees")]
fn set_fees(data: TradingFees) {
    assert_is_manager().unwrap();
    has_trading_fees::init_fee_info(data);
}

#[query(name = "getAccruedFees")]
#[candid_method(update, rename = "getAccruedFees")]
fn get_accrued_fees(data: TradingFees) -> LiquidityAmount {
    STATE.with(|s| s.borrow().deposit_fees.clone())
}
