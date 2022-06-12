use candid::Nat;

use enoki_exchange_shared::has_trading_fees::TradingFees;
use enoki_exchange_shared::types::OrderInput;

fn main() {
    let order = OrderInput {
        allow_taker: true,
        limit_price_in_b: 1.32,
        expiration_time: Some(3_000_000),
    };

    let json = serde_json::to_string(&order).unwrap();
    println!("json: {}", json);

    let retrieved: OrderInput = serde_json::from_str(&json).unwrap();
    println!("retrieved: {:?}", retrieved);

    let fees = TradingFees {
        token_a_deposit_fee: Nat::from(133u32).into(),
        token_b_deposit_fee: Default::default(),
        limit_order_taker_fee: 0.0,
        swap_fee: 0.003,
        swap_market_maker_reward: 0.45,
    };

    let json = serde_json::to_string(&fees).unwrap();
    println!("json: {}", json);

    let retrieved: TradingFees = serde_json::from_str(&json).unwrap();
    println!("retrieved: {:?}", retrieved);
}
