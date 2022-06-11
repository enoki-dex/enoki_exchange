use candid::{CandidType, Deserialize};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_sharded_users::ShardedUserState;
use enoki_exchange_shared::has_token_info::{TokenInfo, TokenInfoState};
use enoki_exchange_shared::has_trading_fees::TradingFees;
use enoki_exchange_shared::is_owned::OwnershipData;
use enoki_exchange_shared::{
    has_sharded_users, has_token_info, has_trading_fees, is_owned,
};
use crate::{brokers, BrokerState, liquidity, orders, synchronize};
use crate::liquidity::LiquidityState;
use crate::orders::OrdersState;

use crate::synchronize::{run, RunningState};

#[derive(Deserialize, CandidType)]
struct UpgradePayload {
    sharded_users: ShardedUserState,
    token_info: TokenInfoState,
    trading_fees: TradingFees,
    owner: OwnershipData,
    orders: OrdersState,
    brokers: BrokerState,
    liquidity: LiquidityState,
    run_state: RunningState,
}

#[pre_upgrade]
fn pre_upgrade() {
    let sharded_users = has_sharded_users::export_stable_storage();
    let token_info = has_token_info::export_stable_storage();
    let trading_fees = has_trading_fees::export_stable_storage();
    let owner = is_owned::export_stable_storage();
    let orders = orders::export_stable_storage();
    let brokers = brokers::export_stable_storage();
    let liquidity = liquidity::export_stable_storage();
    let run_state = synchronize::export_stable_storage();
    let payload = UpgradePayload {
        sharded_users,
        token_info,
        trading_fees,
        owner,
        orders,
        brokers,
        liquidity,
        run_state
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
        trading_fees,
        owner, orders, brokers, liquidity, run_state,
    } = payload;

    has_sharded_users::import_stable_storage(sharded_users);
    has_token_info::import_stable_storage(token_info);
    has_trading_fees::import_stable_storage(trading_fees);
    is_owned::import_stable_storage(owner);
    orders::import_stable_storage(orders);
    brokers::import_stable_storage(brokers);
    liquidity::import_stable_storage(liquidity);
    synchronize::import_stable_storage(run_state);
}
