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
