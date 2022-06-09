use enoki_exchange_shared::types::{OrderInput, Side};
use serde_json::{Result, Value};

fn main() {
    let order = OrderInput {
        allow_taker: true,
        limit_price_in_b: 1.32,
        expiration_time: Some(3_000_000)
    };

    let json = serde_json::to_string(&order).unwrap();
    println!("json: {}", json);

    let retrieved: OrderInput = serde_json::from_str(&json).unwrap();
    println!("retrieved: {:?}", retrieved);
}
