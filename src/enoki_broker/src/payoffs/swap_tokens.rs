use candid::{Nat, Principal};

use enoki_exchange_shared::has_sharded_users::get_user_shard;
use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::types::*;

use crate::token_liquidity_params::{get_lp_worker_assigned_shard, get_lp_worker_location};

pub async fn send_swap_tokens(
    user: Principal,
    token: &EnokiToken,
    amount_to_send: Nat,
) -> Result<()> {
    ic_cdk::api::print(format!(
        "[broker] swap -> sending user {} {:?} {:?}",
        user, token, amount_to_send
    ));
    let lp_location = get_lp_worker_location();
    let lp_shard = get_lp_worker_assigned_shard(token);
    let user_shard = get_user_shard(user, has_token_info::get_token_address(token))?;
    let result: Result<()> = ic_cdk::call(
        lp_shard,
        "shardSpend",
        (lp_location, user_shard, user, amount_to_send),
    )
    .await
    .map_err(|e| e.into_tx_error());
    result
}
