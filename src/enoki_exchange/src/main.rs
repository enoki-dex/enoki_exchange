use candid::{candid_method, Principal, types::number::Nat};
use ic_cdk_macros::*;

#[init]
#[candid_method(init)]
fn init(
) {
    //TODO
}

#[cfg(any(target_arch = "wasm32", test))]
fn main() {}

#[cfg(not(any(target_arch = "wasm32", test)))]
fn main() {
    candid::export_service!();
    std::print!("{}", __export_service());
}

