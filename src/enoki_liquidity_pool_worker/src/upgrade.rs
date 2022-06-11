use candid::{CandidType, Deserialize};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_sharded_users::ShardedUserState;
use enoki_exchange_shared::has_token_info::{TokenInfo, TokenInfoState};
use enoki_exchange_shared::has_trading_fees::TradingFees;
use enoki_exchange_shared::is_managed::ManagementData;
use enoki_exchange_shared::is_owned::OwnershipData;
use enoki_exchange_shared::{
    has_sharded_users, has_token_info, has_trading_fees, is_managed, is_owned,
};
use crate::liquidity;
use crate::liquidity::LiquidityState;


#[derive(Deserialize, CandidType)]
struct UpgradePayload {
    sharded_users: ShardedUserState,
    token_info: TokenInfoState,
    manager: ManagementData,
    owner: OwnershipData,
    liquidity: LiquidityState,
}

#[pre_upgrade]
fn pre_upgrade() {
    let sharded_users = has_sharded_users::export_stable_storage();
    let token_info = has_token_info::export_stable_storage();
    let manager = is_managed::export_stable_storage();
    let owner = is_owned::export_stable_storage();
    let liquidity = liquidity::export_stable_storage();
    let payload = UpgradePayload {
        sharded_users,
        token_info,
        manager,
        owner,
        liquidity
    };
    ic_cdk::storage::stable_save((payload,)).expect("failed to save to stable storage");
}

#[post_upgrade]
fn post_upgrade() {
    let (payload,): (UpgradePayload,) =
        ic_cdk::storage::stable_restore().expect("failed to restore from stable storage");

    let UpgradePayload {
        sharded_users,
        token_info,
        manager,
        owner, liquidity,
    } = payload;

    has_sharded_users::import_stable_storage(sharded_users);
    has_token_info::import_stable_storage(token_info);
    is_managed::import_stable_storage(manager);
    is_owned::import_stable_storage(owner);
    liquidity::import_stable_storage(liquidity);
}
