use std::string::String;

use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::api::call::RejectionCode;

mod serialization;
mod conversion;

#[derive(CandidType, Debug, Deserialize)]
pub enum TxError {
    Unauthorized,
    CallbackError(String),
    Other(String),
}

impl From<(RejectionCode, String)> for TxError {
    fn from(err: (RejectionCode, String)) -> Self {
        Self::CallbackError(format!("Error in callback (code {:?}): {}", err.0, err.1))
    }
}

pub type Result<T> = std::result::Result<T, TxError>;

#[derive(CandidType, Debug, Clone, Default)]
pub struct StableNat(Nat);

#[derive(CandidType, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum EnokiToken {
    TokenA,
    TokenB,
}

#[derive(CandidType, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenAmount {
    token: EnokiToken,
    amount: StableNat,
}

#[derive(CandidType, Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct LiquidityAmount {
    token_a: StableNat,
    token_b: StableNat,
}
