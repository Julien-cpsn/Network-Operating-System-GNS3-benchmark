/// `run_2` < `run_10`: compares the text prefix, then the trailing number.
pub fn natural_key(name: &str) -> (String, u64) {
    let split = name.trim_end_matches(|c: char| c.is_ascii_digit()).len();
    let (prefix, digits) = name.split_at(split);
    (prefix.to_string(), digits.parse().unwrap_or(0))
}
