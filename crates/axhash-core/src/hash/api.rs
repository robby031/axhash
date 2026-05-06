use core::hash::{Hash, Hasher};

use crate::bytes::hash_bytes_core;
use crate::math::{avalanche, seed_lane};

use super::AxHasher;

// Hashes `bytes` using the default seed (`0`).
//
// This is the simplest entry point. For seeded hashing see [`axhash_seeded`].
//
// # Example
//
// ```rust
// use axhash_core::axhash;
//
// let h = axhash(b"hello");
// assert_ne!(h, 0);
// ```
#[inline(always)]
pub fn axhash(bytes: &[u8]) -> u64 {
    axhash_seeded(bytes, 0)
}

// Hashes `bytes` with the given `seed`.
//
// The same `(bytes, seed)` pair always produces the same output. Different
// seeds produce independent hash families — use this for hash-flooding
// resistance or domain separation.
//
// # Example
//
// ```rust
// use axhash_core::axhash_seeded;
//
// let h1 = axhash_seeded(b"hello", 0xdead_beef);
// let h2 = axhash_seeded(b"hello", 0xdead_beef);
// assert_eq!(h1, h2);
//
// let h3 = axhash_seeded(b"hello", 0xcafe_f00d);
// assert_ne!(h1, h3);
// ```
#[inline(always)]
pub fn axhash_seeded(bytes: &[u8], seed: u64) -> u64 {
    avalanche(hash_bytes_core(bytes, seed_lane(seed, 0)))
}

// Hashes any value that implements [`Hash`] using the default seed (`0`).
//
// The output depends on how the type implements `Hash`. For stable,
// reproducible hashes across processes or machines prefer a plain-bytes
// representation and [`axhash`] / [`axhash_seeded`].
//
// # Example
//
// ```rust
// use axhash_core::axhash_of;
//
// #[derive(Hash)]
// struct Point { x: i32, y: i32 }
//
// let h = axhash_of(&Point { x: 1, y: 2 });
// assert_ne!(h, 0);
// ```
#[inline(always)]
pub fn axhash_of<T: Hash>(data: &T) -> u64 {
    axhash_of_seeded(data, 0)
}

// Hashes any value that implements [`Hash`] with the given `seed`.
//
// # Example
//
// ```rust
// use axhash_core::axhash_of_seeded;
//
// #[derive(Hash)]
// struct Point { x: i32, y: i32 }
//
// let h = axhash_of_seeded(&Point { x: 1, y: 2 }, 0xdead_beef);
// assert_eq!(h, axhash_of_seeded(&Point { x: 1, y: 2 }, 0xdead_beef));
// ```
#[inline(always)]
pub fn axhash_of_seeded<T: Hash>(data: &T, seed: u64) -> u64 {
    let mut hasher = AxHasher::new_with_seed(seed);
    data.hash(&mut hasher);
    hasher.finish()
}
