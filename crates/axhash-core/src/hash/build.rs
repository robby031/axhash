use core::hash::BuildHasher;

use crate::constants::SECRET;

use super::AxHasher;

// A [`BuildHasher`] that constructs [`AxHasher`] instances.
//
// Use this type as the hasher factory for `HashMap`, `HashSet`, and any
// other collection that accepts a custom `BuildHasher`.
//
// # Seeding
//
// [`AxBuildHasher::new`] uses an internal constant as the seed. For
// hash-flooding resistance supply your own seed with
// [`AxBuildHasher::with_seed`].
//
// # Example
//
// ```rust
// use std::collections::HashMap;
// use axhash_core::AxBuildHasher;
//
// // Fixed seed — deterministic across runs.
// let mut map: HashMap<&str, u32, AxBuildHasher> =
//     HashMap::with_hasher(AxBuildHasher::with_seed(0xfeed_beef));
// map.insert("alpha", 1);
// map.insert("beta",  2);
// assert_eq!(map["alpha"], 1);
// ```
#[derive(Clone, Copy, Default)]
pub struct AxBuildHasher {
    pub(crate) prepared_seed: u64,
}

impl AxBuildHasher {
    // Creates an `AxBuildHasher` using the library's built-in default seed.
    //
    // The output is deterministic for a given input but the seed value itself
    // is an internal constant and may change between crate versions.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            prepared_seed: SECRET[0],
        }
    }

    // Creates an `AxBuildHasher` with a caller-supplied `seed`.
    //
    // Use a random 64-bit value (e.g. from `rand` or OS entropy) to achieve
    // per-process hash randomisation and resist hash-flooding attacks.
    //
    // # Example
    //
    // ```rust
    // use axhash_core::AxBuildHasher;
    //
    // let bh = AxBuildHasher::with_seed(0x0123_4567_89ab_cdef);
    // ```
    #[inline(always)]
    pub const fn with_seed(seed: u64) -> Self {
        Self {
            prepared_seed: seed ^ SECRET[0],
        }
    }
}

impl BuildHasher for AxBuildHasher {
    type Hasher = AxHasher;

    #[inline(always)]
    fn build_hasher(&self) -> Self::Hasher {
        AxHasher {
            acc: self.prepared_seed,
            sponge: 0,
            sponge_bits: 0,
        }
    }
}
