use ic_cdk_macros::*;

use crate::synchronize::run;

#[heartbeat]
fn tick() {
    ic_cdk::spawn(run())
}