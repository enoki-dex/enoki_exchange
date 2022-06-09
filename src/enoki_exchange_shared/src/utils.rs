use std::collections::HashMap;

use candid::Nat;
use num_traits::{FromPrimitive, ToPrimitive};

use crate::types::{Result, TxError};

pub fn flat_map_vecs<T, V>(vec: Vec<(Vec<T>, Vec<V>)>) -> (Vec<T>, Vec<V>) {
    let total_len: usize = vec.iter().map(|i| i.0.len()).sum();
    let mut left: Vec<T> = Vec::with_capacity(total_len);
    let mut right: Vec<V> = Vec::with_capacity(total_len);
    for (mut l, mut r) in vec {
        left.append(&mut l);
        right.append(&mut r);
    }
    (left, right)
}

pub fn map_assign<T: Clone + Eq + std::hash::Hash, V: Default, TF: Fn(&mut V, V)>(
    map: &mut HashMap<T, V>,
    mut other: HashMap<T, V>,
    assign: TF,
) {
    for key in map.keys().chain(other.keys()).cloned().collect::<Vec<_>>() {
        if let Some(other) = other.remove(&key) {
            assign(map.entry(key).or_default(), other);
        }
    }
}

pub fn nat_x_float(value: Nat, multiplier: f64) -> Result<Nat> {
    let mut val = value.0.to_f64().ok_or(TxError::IntOverflow)?;
    val *= multiplier;
    Ok(Nat::from(
        num_bigint::BigUint::from_f64(val).ok_or(TxError::IntOverflow)?,
    ))
}
