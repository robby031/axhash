use core::hash::BuildHasher;

use crate::constants::SECRET;

use super::AxHasher;

#[derive(Clone, Copy)]
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

    /// Create a new build hasher with a randomized seed drawn from OS entropy.
    /// Panics if the OS entropy source is unavailable.
    #[inline]
    pub fn random() -> Self {
        let mut buf = [0u8; 8];
        match getrandom::getrandom(&mut buf) {
            Ok(()) => Self::with_seed(u64::from_le_bytes(buf)),
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
