use core::hash::BuildHasher;

use crate::math::{DEFAULT_ACC, seed_lane};

use super::AxHasher;

#[derive(Clone, Copy)]
pub struct AxBuildHasher {
    pub(crate) prepared_seed: u64,
}

impl AxBuildHasher {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            prepared_seed: DEFAULT_ACC,
        }
    }

    #[inline(always)]
    pub const fn with_seed(seed: u64) -> Self {
        Self {
            prepared_seed: seed_lane(seed, 0),
        }
    }

    #[inline]
    pub fn random() -> Self {
        match getrandom::u64() {
            Ok(seed) => Self::with_seed(seed),
            Err(e) => panic!("failed to obtain random seed for AxBuildHasher: {e:?}"),
        }
    }
}

impl Default for AxBuildHasher {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
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
