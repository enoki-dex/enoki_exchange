use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::has_token_info::price_in_b_float_to_u64;
use enoki_exchange_shared::interfaces::enoki_wrapped_token::ShardedTransferNotification;
use enoki_exchange_shared::types::*;
use crate::orders::is_user_registered;

use crate::payoffs::charge_deposit_fee;

pub fn validate_order_input(
    notification: ShardedTransferNotification,
    is_swap: bool,
) -> Result<ProcessedOrderInput> {
    let token = has_token_info::parse_from()?;
    let user = notification.from;
    let order: OrderInput = serde_json::from_str(&notification.data)
        .map_err(|e| TxError::ParsingError(e.to_string()))?;
    let quantity = charge_deposit_fee(&token, notification.value)?;
    let side = match &token {
        EnokiToken::TokenA => Side::Sell,
        EnokiToken::TokenB => Side::Buy,
    };
    let price = price_in_b_float_to_u64(order.limit_price_in_b)?;
    if !is_user_registered(user) {
        return Err(TxError::UserNotRegistered {user: user.to_string(), registry: ic_cdk::id().to_string()}.into());
    }

    let order = ProcessedOrderInput {
        user,
        side,
        quantity,
        maker_taker: match (is_swap, order.allow_taker) {
            (true, _) => MakerTaker::OnlyTaker,
            (false, true) => MakerTaker::MakerOrTaker,
            (false, false) => MakerTaker::OnlyMaker,
        },
        limit_price_in_b: price,
        expiration_time: order.expiration_time,
    };
    ic_cdk::println!("[broker] order accepted: {:?}", order);
    Ok(order)
}
