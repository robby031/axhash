#![doc = include_str!("../README.md")]

pub use axhash_core::hash::AxHasher;
pub use axhash_core::hash::api::{axhash, axhash_of, axhash_of_seeded, axhash_seeded};
pub use axhash_core::hash::build::AxBuildHasher;
pub use axhash_core::{RuntimeBackend, runtime_backend, runtime_has_aes};

#[deprecated(since = "0.8.0", note = "use `axhash::AxHasher` directly")]
pub type Hasher = AxHasher;

#[deprecated(since = "0.8.0", note = "use `axhash::AxBuildHasher` directly")]
pub type BuildHasher = AxBuildHasher;

#[inline(always)]
pub fn hash(bytes: &[u8]) -> u64 {
    axhash(bytes)
}

#[inline(always)]
pub fn hash_with_seed(bytes: &[u8], seed: u64) -> u64 {
    axhash_seeded(bytes, seed)
}

#[inline(always)]
pub fn hash_value<T: core::hash::Hash>(value: &T) -> u64 {
    axhash_of(value)
}

#[inline(always)]
pub fn hash_value_with_seed<T: core::hash::Hash>(value: &T, seed: u64) -> u64 {
    axhash_of_seeded(value, seed)
}
