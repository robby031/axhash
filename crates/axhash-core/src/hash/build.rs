use core::hash::BuildHasher;

use crate::constants::SECRET;

use super::AxHasher;

#[derive(Clone, Copy, Default)]
pub struct AxBuildHasher {
    pub(crate) prepared_seed: u64,
}

impl AxBuildHasher {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            prepared_seed: SECRET[0],
        }
    }

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
