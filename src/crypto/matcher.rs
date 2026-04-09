/// Check if a base58-encoded address starts with the given pattern.
/// Case-sensitive, no normalization.
#[inline]
pub fn matches_prefix(address: &str, pattern: &str) -> bool {
    address.starts_with(pattern)
}
