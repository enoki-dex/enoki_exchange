use enoki_exchange_shared::types::{OrderInput, Side};
use serde_json::{Result, Value};

fn main() {
    let order = OrderInput {
        side: Side::Sell,
        allow_taker: true,
        limit_price: "1_000_000_000".to_string(),
        quantity: "3_000_000_000".to_string(),
        expiration_time: Some(3_000_000)
    };

    let json = serde_json::to_string(&order).unwrap();
    println!("json: {}", json);

    let retrieved: OrderInput = serde_json::from_str(&json).unwrap();
    println!("retrieved: {:?}", order);
}
