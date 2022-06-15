use candid::Principal;

use enoki_exchange_shared::has_token_info::get_token_address;
use enoki_exchange_shared::types::{EnokiToken, IntoTxError, Result};

use crate::payoffs::with_valid_token_shards;

pub async fn is_valid_token_shard(token: &EnokiToken, shard: Principal) -> bool {
    if with_valid_token_shards(token, |shards| shards.contains(&shard)) {
        true
    } else {
        update_token_shards(token).await;
        with_valid_token_shards(token, |shards| shards.contains(&shard))
    }
}

async fn update_token_shards(token: &EnokiToken) {
    let response: Result<(Vec<Principal>,)> =
        ic_cdk::call(get_token_address(token), "getShardIdsUpdate", ())
            .await
            .map_err(|e| e.into_tx_error().into());
    let valid_shards = response.unwrap().0;
    with_valid_token_shards(token, |shards| *shards = valid_shards.into_iter().collect());
}
