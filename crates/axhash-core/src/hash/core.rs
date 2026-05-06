use crate::constants::SECRET;
use crate::math::folded_multiply;

// A streaming hasher that implements [`std::hash::Hasher`].
//
// `AxHasher` accumulates typed values (primitives, byte slices) into an
// internal accumulator. Call [`finish`](core::hash::Hasher::finish) to obtain
// the final `u64` digest. The hasher can be reused by constructing a new
// instance; there is no reset method because construction is cheap.
//
// # Usage with `HashMap`
//
// Use [`AxBuildHasher`](super::build::AxBuildHasher) as the `BuildHasher` for
// standard collections:
//
// ```rust
// use std::collections::HashMap;
// use axhash_core::AxBuildHasher;
//
// let mut map: HashMap<&str, u32, AxBuildHasher> =
//     HashMap::with_hasher(AxBuildHasher::new());
// map.insert("hello", 42);
// ```
//
// # Streaming example
//
// ```rust
// use axhash_core::AxHasher;
// use std::hash::Hasher as _;
//
// let mut h = AxHasher::new_with_seed(0x1234);
// h.write(b"part-1");
// h.write(b"part-2");
// let digest = h.finish();
// assert_ne!(digest, 0);
// ```
#[derive(Clone, Debug)]
pub struct AxHasher {
    pub(crate) acc: u64,
    pub(crate) sponge: u128,
    pub(crate) sponge_bits: u8,
}

impl AxHasher {
    // Creates a new `AxHasher` with the default seed (`0`).
    #[inline(always)]
    pub fn new() -> Self {
        Self::new_with_seed(0)
    }

    // Creates a new `AxHasher` initialised with the given `seed`.
    //
    // Different seeds produce independent hash families. This is useful for
    // hash-flooding resistance or domain separation between hasher instances.
    #[inline(always)]
    pub fn new_with_seed(seed: u64) -> Self {
        Self {
            acc: seed ^ SECRET[0],
            sponge: 0,
            sponge_bits: 0,
        }
    }

    #[inline(always)]
    pub(crate) fn flush_sponge(&mut self) {
        if self.sponge_bits == 0 {
            return;
        }

        let lo = self.sponge as u64;
        let hi = (self.sponge >> 64) as u64;
        self.acc = folded_multiply(lo ^ self.acc, hi ^ SECRET[1]);
        self.sponge = 0;
        self.sponge_bits = 0;
    }

    #[inline(always)]
    pub(crate) fn push_num<T: Into<u128>>(&mut self, value: T, bits: u8) {
        if self.sponge_bits as u16 + bits as u16 > 128 {
            self.flush_sponge();
        }

        self.sponge |= value.into() << self.sponge_bits;
        self.sponge_bits += bits;
    }
}

impl Default for AxHasher {
    // Returns a default `AxHasher` equivalent to [`AxHasher::new`].
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}
