pub use anyhow::Result;
use candid::{CandidType, Deserialize};
use ic_cdk::api::call::RejectionCode;
use thiserror::Error;

use crate::types::EnokiToken;

#[derive(CandidType, Debug, Deserialize, Error)]
pub enum TxError {
    #[error("Insufficient Funds in {token:?}: funds={funds} vs needed={needed}")]
    InsufficientFunds {
        token: EnokiToken,
        funds: String,
        needed: String,
    },
    #[error("Insufficient Liquidity Available")]
    InsufficientLiquidityAvailable,
    #[error("Slippage Exceeded (limit {limit_price} vs actual {actual_price}): swap was cancelled")]
    SlippageExceeded {limit_price: u64, actual_price: u64},
    #[error("Unauthorized")]
    Unauthorized,
    #[error("User {user} Not Registered at {registry}")]
    UserNotRegistered { user: String, registry: String },
    #[error("Internal error: int overflow")]
    IntOverflow,
    #[error("Internal error: int underflow")]
    IntUnderflow,
    #[error("Quantity too low")]
    QuantityTooLow,
    #[error("Cannot subtract a larger uint from a smaller one.")]
    UIntSubtractError,
    #[error("Parsing error: {0}")]
    ParsingError(String),
    #[error("Callback error: {0}")]
    CallbackError(String),
    #[error("Error: {0}")]
    Other(String),
}

pub trait IntoTxError {
    fn into_tx_error(self) -> anyhow::Error;
}

impl IntoTxError for (RejectionCode, String) {
    fn into_tx_error(self) -> anyhow::Error {
        TxError::CallbackError(format!("Error in callback (code {:?}): {}", self.0, self.1)).into()
    }
}
