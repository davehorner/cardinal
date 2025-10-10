// src/util.rs
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Non-cryptographic, stable hash for cache bucketing.
pub fn hash_url(url: &str) -> u64 {
    let mut h = DefaultHasher::new();
    url.hash(&mut h);
    h.finish()
}
