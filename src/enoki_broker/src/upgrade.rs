use candid::{CandidType, Deserialize};
use ic_cdk_macros::*;

use enoki_exchange_shared::{
    has_sharded_users, has_token_info, has_trading_fees, is_managed, is_owned,
};
use enoki_exchange_shared::has_sharded_users::ShardedUserState;
use enoki_exchange_shared::has_token_info::TokenInfoState;
use enoki_exchange_shared::has_trading_fees::TradingFees;
use enoki_exchange_shared::is_managed::ManagementData;
use enoki_exchange_shared::is_owned::OwnershipData;

use crate::{liquidity, orders, other_brokers, payoffs, token_liquidity_params};
use crate::liquidity::LiquidityState;
use crate::orders::OrdersState;
use crate::other_brokers::BrokersState;
use crate::payoffs::{AccruedFees, PayoffsState};
use crate::token_liquidity_params::TokenLiquidityData;

#[derive(Deserialize, CandidType)]
struct UpgradePayload {
    liquidity: LiquidityState,
    brokers: BrokersState,
    token_liquidity_params: TokenLiquidityData,
    fees: AccruedFees,
    payoffs: PayoffsState,
    orders: OrdersState,
    sharded_users: ShardedUserState,
    token_info: TokenInfoState,
    trading_fees: TradingFees,
    manager: ManagementData,
    owner: OwnershipData,
}

#[pre_upgrade]
fn pre_upgrade() {
    let liquidity = liquidity::export_stable_storage();
    let brokers = other_brokers::export_stable_storage();
    let token_liquidity_params = token_liquidity_params::export_stable_storage();
    let fees = payoffs::export_stable_storage_fees();
    let payoffs = payoffs::export_stable_storage();
    let orders = orders::export_stable_storage();
    let sharded_users = has_sharded_users::export_stable_storage();
    let token_info = has_token_info::export_stable_storage();
    let trading_fees = has_trading_fees::export_stable_storage();
    let manager = is_managed::export_stable_storage();
    let owner = is_owned::export_stable_storage();
    let payload = UpgradePayload {
        liquidity,
        brokers,
        token_liquidity_params,
        fees,
        payoffs,
        orders,
        sharded_users,
        token_info,
        trading_fees,
        manager,
        owner,
    };
    ic_cdk::storage::stable_save((payload, )).expect("failed to save to stable storage");
}

#[post_upgrade]
fn post_upgrade() {
    let (payload, ): (UpgradePayload, ) =
        ic_cdk::storage::stable_restore().expect("failed to restore from stable storage");

    let UpgradePayload {
        liquidity,
        brokers,
        token_liquidity_params,
        fees,
        payoffs,
        orders,
        sharded_users,
        token_info,
        trading_fees,
        manager,
        owner,
    } = payload;

    liquidity::import_stable_storage(liquidity);
    other_brokers::import_stable_storage(brokers);
    token_liquidity_params::import_stable_storage(token_liquidity_params);
    payoffs::import_stable_storage_fees(fees);
    payoffs::import_stable_storage(payoffs);
    orders::import_stable_storage(orders);
    has_sharded_users::import_stable_storage(sharded_users);
    has_token_info::import_stable_storage(token_info);
    has_trading_fees::import_stable_storage(trading_fees);
    is_managed::import_stable_storage(manager);
    is_owned::import_stable_storage(owner);
}
