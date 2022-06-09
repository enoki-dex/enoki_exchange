use std::collections::{BTreeMap, HashMap};

use candid::{candid_method, CandidType, Nat, Principal};

use enoki_exchange_shared::types::*;

pub trait OrderMatching {
    fn try_execute(&mut self, other: &mut Self);
    fn is_complete(&self) -> bool;
}

impl OrderMatching for Order {
    fn try_execute(&mut self, executor: &mut Self) {
        if let OrderStatus::Pending = self.state.status {
            if let OrderStatus::Pending = executor.state.status {
                let quantity_traded = self
                    .state
                    .quantity_remaining
                    .clone()
                    .min(executor.state.quantity_remaining.clone());
                self.state.quantity_remaining -= quantity_traded.clone();
                executor.state.quantity_remaining -= quantity_traded;
                if !self.state.quantity_remaining.is_nonzero() {
                    self.state.status = OrderStatus::Completed;
                }
                if !executor.state.quantity_remaining.is_nonzero() {
                    executor.state.status = OrderStatus::Completed;
                }
                self.state.marker_makers.push((&*executor).into());
            }
        }
    }
    fn is_complete(&self) -> bool {
        if let OrderStatus::Pending = self.state.status {
            false
        } else {
            true
        }
    }
}
