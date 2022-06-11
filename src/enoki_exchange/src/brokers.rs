use std::cell::RefCell;
use std::collections::HashMap;

use candid::{candid_method, CandidType, Nat, Principal};
use candid::utils::{ArgumentDecoder, ArgumentEncoder};
use ic_cdk_macros::*;

use enoki_exchange_shared::{has_token_info, has_trading_fees};
use enoki_exchange_shared::has_sharded_users::{get_user_shard, register_user};
use enoki_exchange_shared::has_token_info::AssignedShards;
use enoki_exchange_shared::has_trading_fees::TradingFees;
use enoki_exchange_shared::is_owned;
use enoki_exchange_shared::types::*;

use crate::liquidity::{self, get_liquidity_location, init_broker_lp};

#[allow(unused)]
pub fn assert_is_broker_contract() -> Result<()> {
    if STATE.with(|s| s.borrow().brokers.contains_key(&ic_cdk::caller())) {
        Ok(())
    } else {
        Err(TxError::Unauthorized)
    }
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug)]
pub struct Broker {
    pub id: Principal,
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct BrokerState {
    pub brokers: HashMap<Principal, Broker>,
}

thread_local! {
    static STATE: RefCell<BrokerState> = RefCell::new(BrokerState::default());
}

pub async fn foreach_broker<
    T: ArgumentEncoder,
    R: for<'a> ArgumentDecoder<'a>,
    TF: FnMut(Principal) -> T,
>(
    method: &str,
    mut args_getter: TF,
) -> Result<Vec<R>> {
    let ids = get_broker_ids();
    let responses: Vec<std::result::Result<R, _>> = futures::future::join_all(
        ids.into_iter()
            .map(|id| ic_cdk::call(id, method, args_getter(id))),
    )
    .await;
    responses
        .into_iter()
        .collect::<std::result::Result<Vec<R>, _>>()
        .map_err(|err| err.into())
}

pub async fn foreach_broker_map<
    T: ArgumentEncoder,
    R: for<'a> ArgumentDecoder<'a>,
    TF: FnMut(Principal) -> T,
    FR,
    RM: Fn(R) -> FR,
>(
    method: &str,
    args_getter: TF,
    results_mapper: RM,
) -> Result<HashMap<Principal, FR>> {
    let ids = get_broker_ids();
    Ok(foreach_broker(method, args_getter)
        .await?
        .into_iter()
        .enumerate()
        .map(|(i, res)| (ids[i], results_mapper(res)))
        .collect())
}

#[query(name = "getBrokers")]
#[candid_method(query, rename = "getBrokers")]
fn get_brokers() -> BrokerState {
    STATE.with(|d| d.borrow().clone())
}

#[query(name = "getBrokerIds")]
#[candid_method(query, rename = "getBrokerIds")]
pub fn get_broker_ids() -> Vec<Principal> {
    let mut ids: Vec<Principal> = STATE.with(|s| s.borrow().brokers.keys().copied().collect());
    ids.sort();
    ids
}

#[update(name = "addBroker")]
#[candid_method(update, rename = "addBroker")]
async fn add_broker(broker: Principal) -> Result<()> {
    is_owned::assert_is_owner()?;
    let token_info = has_token_info::get_token_info();
    let token_a = token_info.token_a.principal;
    let token_b = token_info.token_b.principal;
    let response: Result<(AssignedShards,)> = ic_cdk::call(
        broker,
        "initBroker",
        (
            get_broker_ids(),
            token_info,
            get_liquidity_location(),
            has_trading_fees::get_trading_fees(),
        ),
    )
    .await
    .map_err(|e| e.into());
    let assigned = response?.0;
    let _result: Vec<()> = foreach_broker("addBroker", |_| (broker,)).await?;
    STATE.with(|s| s.borrow_mut().brokers.insert(broker, Broker { id: broker }));
    register_user(broker, token_a, assigned.token_a);
    register_user(broker, token_b, assigned.token_b);
    init_broker_lp(broker);

    let result: Result<()> = ic_cdk::call(liquidity::get_pool_contract(), "addBroker", (broker,))
        .await
        .map_err(|e| e.into());
    result?;

    Ok(())
}

#[allow(unused)]
pub fn get_broker_shard(broker: Principal, token: &EnokiToken) -> Result<Principal> {
    get_user_shard(broker, has_token_info::get_token_address(token))
}

pub fn export_stable_storage() -> BrokerState {
    STATE.with(|b| b.take())
}

pub fn import_stable_storage(data: BrokerState) {
    STATE.with(|b| b.replace(data));
}

#[update(name = "setFees")]
#[candid_method(update, rename = "setFees")]
async fn set_fees(
    token_a_deposit_fee: Nat,
    token_b_deposit_fee: Nat,
    limit_order_taker_fee: f64,
    swap_fee: f64,
    swap_market_maker_reward: f64,
) {
    is_owned::assert_is_owner().unwrap();
    has_trading_fees::init_fee_info(TradingFees {
        token_a_deposit_fee: token_a_deposit_fee.into(),
        token_b_deposit_fee: token_b_deposit_fee.into(),
        limit_order_taker_fee,
        swap_fee,
        swap_market_maker_reward,
    });
    let data = has_trading_fees::get_trading_fees();
    let _result: Vec<()> = foreach_broker("setFees", |_| (data.clone(),))
        .await
        .unwrap();
}
