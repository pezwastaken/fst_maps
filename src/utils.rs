//get the index of the last prefix character inside y
pub(crate) fn common_prefix_index(previous_str: &[u8], y: &[u8]) -> usize {
    //TODO SIMD it using chunk_exact
    let prefix_len = previous_str
        .iter()
        .zip(y)
        .take_while(|(x, y)| x == y)
        .count();

    assert!(prefix_len <= y.len());
    return prefix_len;
}
