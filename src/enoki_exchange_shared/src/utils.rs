use std::collections::HashMap;
use std::ops::{AddAssign, SubAssign};

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

pub fn map_assign<T: Clone + Eq + std::hash::Hash, V: SubAssign + Default, TF: Fn(&mut V, V)>(map: &mut HashMap<T, V>, mut other: HashMap<T, V>, assign: TF) {
    for key in map.keys().chain(other.keys()).cloned().collect::<Vec<_>>() {
        if let Some(other) = other.remove(&key) {
            assign(map.entry(key).or_default(), other);
        }
    }
}
