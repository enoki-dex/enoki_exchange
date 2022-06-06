use candid::{candid_method, Nat, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_token_info::AssignedShards;
use enoki_exchange_shared::is_owned;
#[allow(unused_imports)]
use enoki_exchange_shared::is_owned::OwnershipData;
#[allow(unused_imports)]
use enoki_exchange_shared::{has_token_info, has_token_info::TokenInfo, types::*};
#[allow(unused_imports)]
use workers::WorkerContractData;

mod exchange_information;
mod liquidity;
mod workers;

#[init]
#[candid_method(init)]
async fn init(owner: Principal, exchange: Principal, token_a: Principal, token_b: Principal) {
    is_owned::init_owner(OwnershipData {
        owner,
        deploy_time: ic_cdk::api::time(),
    });
    exchange_information::init_exchange_information(exchange);
    let token_info = TokenInfo { token_a, token_b };
    let (assigned_a, assigned_b) = register_tokens(&token_info).await.unwrap();
    has_token_info::init_token_info(
        token_info,
        AssignedShards {
            token_a: assigned_a,
            token_b: assigned_b,
        },
    );
}

async fn register_tokens(token_info: &TokenInfo) -> Result<(Principal, Principal)> {
    let (assigned_a, assigned_b) = tokio::join!(
        has_token_info::register(token_info.token_a),
        has_token_info::register(token_info.token_b)
    );
    Ok((assigned_a?, assigned_b?))
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}
