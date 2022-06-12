use candid::candid_method;
use ic_cdk_macros::*;

use crate::synchronize::do_run;

//TODO: enable
// #[heartbeat]
// fn tick() {
//     ic_cdk::spawn(run())
// }

#[update(name = "triggerRun")]
#[candid_method(update, rename = "triggerRun")]
async fn trigger_run() {
    do_run().await.unwrap()
}
