use core::hash::Hash;
use std::hash::Hasher;

use crate::bytes::hash_bytes_core;
use crate::math::{avalanche, seed_lane};

use super::AxHasher;

#[inline(always)]
pub fn axhash(bytes: &[u8]) -> u64 {
    axhash_seeded(bytes, 0)
}

#[inline(always)]
pub fn axhash_seeded(bytes: &[u8], seed: u64) -> u64 {
    avalanche(hash_bytes_core(bytes, seed_lane(seed, 0)))
}

#[inline(always)]
pub fn axhash_of<T: Hash>(data: &T) -> u64 {
    axhash_of_seeded(data, 0)
}

#[inline(always)]
pub fn axhash_of_seeded<T: Hash>(data: &T, seed: u64) -> u64 {
    let mut hasher = AxHasher::new_with_seed(seed);
    data.hash(&mut hasher);
    hasher.finish()
}
