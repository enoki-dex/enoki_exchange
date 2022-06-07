use candid::{CandidType, Deserialize, Principal, Nat};

#[derive(CandidType, Debug, Deserialize)]
pub struct ShardedTransferNotification {
    pub from: Principal,
    pub from_shard: Principal,
    pub to: Principal,
    pub value: Nat,
}