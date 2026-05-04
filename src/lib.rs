pub use axhash_core::{
    AxBuildHasher,
    AxHasher,
    RuntimeBackend,
    axhash,
    axhash_of,
    axhash_of_seeded,
    axhash_seeded,
    runtime_backend,
    runtime_has_aes,
};

pub type Hasher = AxHasher;
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
