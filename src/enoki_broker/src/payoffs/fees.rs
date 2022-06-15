use std::cell::RefCell;
use std::cmp::Ordering;
use std::ops::AddAssign;

use candid::{candid_method, CandidType, Nat};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::has_trading_fees;
use enoki_exchange_shared::has_trading_fees::{get_deposit_fee, TradingFees};
use enoki_exchange_shared::is_managed::assert_is_manager;
use enoki_exchange_shared::is_owned::assert_is_owner;
use enoki_exchange_shared::types::*;

thread_local! {
    static STATE: RefCell<AccruedFees> = RefCell::new(AccruedFees::default());
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct AccruedFees {
    deposit_fees: LiquidityAmount,
    token_a_transfer_fee: Option<StableNat>,
    token_b_transfer_fee: Option<StableNat>,
}

impl AccruedFees {
    pub fn get_token_fee(&self, token: &EnokiToken) -> Option<Nat> {
        match token {
            EnokiToken::TokenA => self.token_a_transfer_fee.clone().map(|val| val.into()),
            EnokiToken::TokenB => self.token_b_transfer_fee.clone().map(|val| val.into()),
        }
    }
    pub fn get_token_fee_mut(&mut self, token: &EnokiToken) -> &mut Option<StableNat> {
        match token {
            EnokiToken::TokenA => &mut self.token_a_transfer_fee,
            EnokiToken::TokenB => &mut self.token_b_transfer_fee,
        }
    }
}

pub fn charge_deposit_fee(token: &EnokiToken, deposit_amount: Nat) -> Result<Nat> {
    let fee = get_deposit_fee(token);
    if deposit_amount <= fee {
        return Err(TxError::QuantityTooLow.into());
    }
    let remaining = deposit_amount - fee.clone();
    STATE.with(|s| {
        s.borrow_mut()
            .deposit_fees
            .get_mut(&token)
            .add_assign(fee.into())
    });
    Ok(remaining)
}

pub fn try_get_fee_for_transfer(token: &EnokiToken) -> Option<Nat> {
    STATE.with(|s| s.borrow().get_token_fee(token))
}

pub async fn get_fee_for_transfer(token: &EnokiToken) -> Result<Nat> {
    if let Some(fee) = try_get_fee_for_transfer(token) {
        Ok(fee)
    } else {
        update_upstream_token_fee(token).await?;
        try_get_fee_for_transfer(token).ok_or(
            TxError::Other("cannot calculate upstream token transfer fee".to_string()).into(),
        )
    }
}

pub async fn use_fee_for_transfer(token: &EnokiToken) -> Result<Nat> {
    let transfer_fee = if let Some(fee) = try_get_fee_for_transfer(token) {
        fee
    } else {
        get_fee_for_transfer(token).await?
    };
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if s.deposit_fees.get(token).compare_with(&transfer_fee) == Ordering::Less {
            Err(TxError::InsufficientFunds {
                token: token.clone(),
                funds: s.deposit_fees.get(token).clone().to_nat().to_string(),
                needed: transfer_fee.to_string(),
            }
            .into())
        } else {
            s.deposit_fees
                .get_mut(token)
                .safe_sub_assign(transfer_fee.clone().into())
                .unwrap();
            Ok(transfer_fee)
        }
    })
}

#[update(name = "setFees")]
#[candid_method(update, rename = "setFees")]
fn set_fees(data: TradingFees) {
    assert_is_manager().unwrap();
    has_trading_fees::init_fee_info(data);
}

#[update(name = "updateUpstreamFees")]
#[candid_method(update, rename = "updateUpstreamFees")]
async fn update_upstream_fees() {
    assert_is_owner().unwrap();
    update_upstream_token_fee(&EnokiToken::TokenA)
        .await
        .unwrap();
    update_upstream_token_fee(&EnokiToken::TokenB)
        .await
        .unwrap();
}

async fn update_upstream_token_fee(token: &EnokiToken) -> Result<()> {
    let result: Result<(Nat,)> =
        ic_cdk::call(has_token_info::get_token_address(token), "getFee", ())
            .await
            .map_err(|e| e.into_tx_error());
    let fee = result?.0;
    STATE.with(|s| *s.borrow_mut().get_token_fee_mut(token) = Some(fee.into()));
    Ok(())
}

#[query(name = "getAccruedFees")]
#[candid_method(query, rename = "getAccruedFees")]
fn get_accrued_fees() -> LiquidityAmount {
    STATE.with(|s| s.borrow().deposit_fees.clone())
}

pub fn export_stable_storage() -> AccruedFees {
    let data = STATE.with(|s| s.take());
    data
}

pub fn import_stable_storage(data: AccruedFees) {
    STATE.with(|s| s.replace(data));
}
