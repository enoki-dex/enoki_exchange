use candid::{CandidType, Deserialize};
use ic_cdk_macros::*;

use crate::liquidity;
use crate::liquidity::LiquidityState;

#[derive(Deserialize, CandidType)]
struct UpgradePayload {
    liquidity: LiquidityState,
}

#[pre_upgrade]
fn pre_upgrade() {
    let liquidity = liquidity::export_stable_storage();
    let payload = UpgradePayload { liquidity };
    ic_cdk::storage::stable_save((payload,)).expect("failed to save to stable storage");
}

#[post_upgrade]
fn post_upgrade() {
    let (payload,): (UpgradePayload,) =
        ic_cdk::storage::stable_restore().expect("failed to restore from stable storage");

    let UpgradePayload { liquidity } = payload;

    liquidity::import_stable_storage(liquidity);
}
