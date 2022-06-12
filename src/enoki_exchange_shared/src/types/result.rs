pub use anyhow::Result;
use candid::{CandidType, Deserialize};
use ic_cdk::api::call::RejectionCode;
use thiserror::Error;

#[derive(CandidType, Debug, Deserialize, Error)]
pub enum TxError {
    #[error("Insufficient Funds")]
    InsufficientFunds,
    #[error("Insufficient Liquidity Available")]
    InsufficientLiquidityAvailable,
    #[error("Slippage Exceeded: swap was cancelled")]
    SlippageExceeded,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("User Not Registered")]
    UserNotRegistered,
    #[error("Internal error: int overflow")]
    IntOverflow,
    #[error("Internal error: int underflow")]
    IntUnderflow,
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
