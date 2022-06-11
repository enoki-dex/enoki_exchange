use enoki_exchange_shared::has_sharded_users::register_user;
use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::has_token_info::price_in_b_float_to_u64;
use enoki_exchange_shared::interfaces::enoki_wrapped_token::ShardedTransferNotification;
use enoki_exchange_shared::types::*;

use crate::payoffs::charge_deposit_fee;

pub fn validate_order_input(
    notification: ShardedTransferNotification,
    is_swap: bool,
) -> Result<ProcessedOrderInput> {
    let token = has_token_info::parse_from()?;
    let user = notification.from;
    let order: OrderInput = serde_json::from_str(&notification.data)
        .map_err(|e| TxError::ParsingError(e.to_string()))?;
    let quantity = charge_deposit_fee(&token, notification.value);
    let side = match &token {
        EnokiToken::TokenA => Side::Sell,
        EnokiToken::TokenB => Side::Buy,
    };
    if quantity == 0u32 {
        return Err(TxError::IntUnderflow);
    }
    let price = price_in_b_float_to_u64(order.limit_price_in_b)?;
    register_user(
        user,
        has_token_info::get_token_address(&token),
        notification.from_shard,
    );

    Ok(ProcessedOrderInput {
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
    })
}
