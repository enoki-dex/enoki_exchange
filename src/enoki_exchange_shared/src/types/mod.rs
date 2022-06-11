use std::collections::BTreeMap;
use std::string::String;

use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::api::call::RejectionCode;

mod implementations;
mod serialization;

#[derive(CandidType, Debug, Deserialize)]
pub enum TxError {
    InsufficientFunds,
    InsufficientLiquidityAvailable,
    SlippageExceeded,
    Unauthorized,
    UserNotRegistered,
    IntOverflow,
    IntUnderflow,
    ParsingError(String),
    CallbackError(String),
    Other(String),
}

impl From<(RejectionCode, String)> for TxError {
    fn from(err: (RejectionCode, String)) -> Self {
        Self::CallbackError(format!("Error in callback (code {:?}): {}", err.0, err.1))
    }
}

pub type Result<T> = std::result::Result<T, TxError>;

#[derive(CandidType, Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct StableNat(pub Nat);

#[derive(CandidType, Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq, Hash)]
pub enum EnokiToken {
    TokenA,
    TokenB,
}

#[derive(CandidType, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenAmount {
    pub token: EnokiToken,
    pub amount: StableNat,
}

#[derive(CandidType, Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct LiquidityAmount {
    pub token_a: StableNat,
    pub token_b: StableNat,
}

#[derive(CandidType, Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct LiquidityTrades {
    pub increased: LiquidityAmount,
    pub decreased: LiquidityAmount,
}

#[derive(CandidType, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(CandidType, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum MakerTaker {
    OnlyMaker,
    OnlyTaker,
    MakerOrTaker,
}

#[derive(CandidType, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrderInfo {
    pub broker: Principal,
    pub user: Principal,
    pub id: u64, // only unique with respect to a broker
    pub side: Side,
    pub maker_taker: MakerTaker,
    pub limit_price: u64,
    pub quantity: StableNat,
    pub expiration_time: Option<u64>,
}

impl Default for OrderInfo {
    fn default() -> Self {
        Self {
            broker: Principal::anonymous(),
            user: Principal::anonymous(),
            id: 0,
            side: Side::Buy,
            maker_taker: MakerTaker::MakerOrTaker,
            limit_price: Default::default(),
            quantity: Default::default(),
            expiration_time: None,
        }
    }
}

#[derive(CandidType, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrderInput {
    pub allow_taker: bool,
    pub limit_price_in_b: f64,
    pub expiration_time: Option<u64>,
}

#[derive(CandidType, Debug, Clone)]
pub struct ProcessedOrderInput {
    pub user: Principal,
    pub side: Side,
    pub quantity: Nat,
    pub maker_taker: MakerTaker,
    pub limit_price_in_b: u64,
    pub expiration_time: Option<u64>,
}

#[derive(CandidType, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum OrderStatus {
    Pending,
    Cancelled,
    Completed,
    Expired,
    InsufficientLiquidity,
    InvalidPrice,
}

#[derive(CandidType, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrderState {
    pub status: OrderStatus,
    pub quantity_remaining: StableNat,
    pub marker_makers: Vec<CounterpartyInfo>,
}

#[derive(CandidType, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Order {
    pub info: OrderInfo,
    pub state: OrderState,
}

#[derive(CandidType, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CounterpartyInfo {
    pub broker: Principal,
    pub user: Principal,
    pub quantity: StableNat,
    pub price: u64,
}

#[derive(CandidType, Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct AggregateBidAsk {
    pub bids: BTreeMap<u64, Vec<CounterpartyInfo>>,
    pub asks: BTreeMap<u64, Vec<CounterpartyInfo>>,
}

#[derive(CandidType, Debug, Clone, Default)]
pub struct BidAskCurve {
    pub num_decimals: u64,
    pub bids: BTreeMap<u64, Nat>,
    pub asks: BTreeMap<u64, Nat>,
}

#[derive(CandidType)]
pub struct OpenOrderStatus {
    pub open_orders: Vec<OrderInfo>,
    pub pending_cancel: Vec<u64>,
}

#[derive(CandidType, Eq, PartialEq, Hash)]
pub struct BrokerAndUser {
    pub broker: Principal,
    pub user: Principal,
}
