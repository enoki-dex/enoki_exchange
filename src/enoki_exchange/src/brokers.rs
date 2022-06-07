use std::cell::RefCell;
use std::collections::HashMap;

use candid::utils::{ArgumentDecoder, ArgumentEncoder};
use candid::{candid_method, CandidType, Deserialize, Principal};
use ic_cdk_macros::*;

use enoki_exchange_shared::has_sharded_users::{get_user_shard, register_user};
use enoki_exchange_shared::has_token_info;
use enoki_exchange_shared::is_owned;
use enoki_exchange_shared::types::*;

pub fn assert_is_broker_contract() -> Result<()> {
    if STATE.with(|s| s.borrow().brokers.contains_key(&ic_cdk::caller())) {
        Ok(())
    } else {
        Err(TxError::Unauthorized)
    }
}

#[derive(Deserialize, CandidType, Clone, Debug)]
pub struct Broker {
    pub id: Principal,
}

#[derive(Deserialize, CandidType, Clone, Debug, Default)]
pub struct BrokerState {
    pub brokers: HashMap<Principal, Broker>,
}

thread_local! {
    static STATE: RefCell<BrokerState> = RefCell::new(BrokerState::default());
}

pub async fn foreach_broker<T: ArgumentEncoder + Clone, R: for<'a> ArgumentDecoder<'a>>(
    method: &str,
    args: T,
) -> Result<Vec<R>> {
    let ids = get_broker_ids();
    let responses: Vec<std::result::Result<R, _>> = futures::future::join_all(
        ids.into_iter()
            .map(|id| ic_cdk::call(id, method, args.clone())),
    )
    .await;
    responses
        .into_iter()
        .collect::<std::result::Result<Vec<R>, _>>()
        .map_err(|err| err.into())
}

#[query(name = "getBrokers")]
#[candid_method(query, rename = "getBrokers")]
fn get_brokers() -> BrokerState {
    STATE.with(|d| d.borrow().clone())
}

#[query(name = "getBrokerIds")]
#[candid_method(query, rename = "getBrokerIds")]
fn get_broker_ids() -> Vec<Principal> {
    STATE.with(|s| s.borrow().brokers.keys().copied().collect())
}

#[update(name = "addBroker")]
#[candid_method(update, rename = "addBroker")]
async fn add_broker(broker: Principal) -> Result<()> {
    is_owned::assert_is_owner()?;
    let response: Result<(Principal,)> =
        ic_cdk::call(broker, "initBroker", (has_token_info::get_token_info(),))
            .await
            .map_err(|e| e.into());
    let shard = response?.0;
    STATE.with(|s| s.borrow_mut().brokers.insert(broker, Broker { id: broker }));
    register_user(ShardedPrincipal {
        shard,
        principal: broker,
    });
    Ok(())
}

pub fn get_broker_shard(broker: Principal) -> Result<Principal> {
    get_user_shard(broker)
}

pub fn export_stable_storage() -> (BrokerState,) {
    let data: BrokerState = STATE.with(|b| b.take());
    (data,)
}

pub fn import_stable_storage(data: BrokerState) {
    STATE.with(|b| b.replace(data));
}
