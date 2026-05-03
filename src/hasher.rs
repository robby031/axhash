use crate::bytes::hash_bytes_core;
use crate::constants::SECRET;
use crate::math::{avalanche, folded_multiply, seed_lane};
use core::hash::{BuildHasher, Hash, Hasher};

#[derive(Clone)]
pub struct AxHasher {
    acc: u64,
    sponge: u128,
    sponge_bits: u8,
}

impl AxHasher {
    #[inline(always)]
    pub fn new() -> Self {
        Self::new_with_seed(0)
    }

    #[inline(always)]
    pub fn new_with_seed(seed: u64) -> Self {
        Self {
            acc: seed ^ SECRET[0],
            sponge: 0,
            sponge_bits: 0,
        }
    }

    #[inline(always)]
    fn flush_sponge(&mut self) {
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
    fn push_num<T: Into<u128>>(&mut self, value: T, bits: u8) {
        if self.sponge_bits as u16 + bits as u16 > 128 {
            self.flush_sponge();
        }

        self.sponge |= value.into() << self.sponge_bits;
        self.sponge_bits += bits;
    }
}

impl Default for AxHasher {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

// AxBuildHasher (Router untuk HashMap)
#[derive(Clone, Copy, Default)]
pub struct AxBuildHasher {
    prepared_seed: u64,
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

// Hash raw bytes non seed eksplisit (default seed = 0)
#[inline(always)]
pub fn axhash(bytes: &[u8]) -> u64 {
    axhash_seeded(bytes, 0)
}

// Hash raw bytes dengan custom seed
#[inline(always)]
pub fn axhash_seeded(bytes: &[u8], seed: u64) -> u64 {
    avalanche(hash_bytes_core(bytes, seed_lane(seed, 0)))
}

// Hash non seed eksplisit
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

impl Hasher for AxHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        if self.sponge_bits == 0 {
            avalanche(self.acc)
        } else {
            let lo = self.sponge as u64;
            let hi = (self.sponge >> 64) as u64;
            avalanche(folded_multiply(lo ^ self.acc, hi ^ SECRET[1]))
        }
    }

    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        self.flush_sponge();
        self.acc = hash_bytes_core(bytes, self.acc);
    }

    #[inline(always)]
    fn write_u8(&mut self, i: u8) {
        self.push_num(i, 8);
    }

    #[inline(always)]
    fn write_u16(&mut self, i: u16) {
        self.push_num(i, 16);
    }

    #[inline(always)]
    fn write_u32(&mut self, i: u32) {
        self.push_num(i, 32);
    }

    #[inline(always)]
    fn write_u64(&mut self, i: u64) {
        self.push_num(i, 64);
    }

    #[inline(always)]
    fn write_u128(&mut self, i: u128) {
        self.flush_sponge();
        let lo = i as u64;
        let hi = (i >> 64) as u64;
        self.acc = folded_multiply(lo ^ self.acc, hi ^ SECRET[1]);
    }

    #[inline(always)]
    fn write_usize(&mut self, i: usize) {
        #[cfg(target_pointer_width = "32")]
        self.write_u32(i as u32);
        #[cfg(target_pointer_width = "64")]
        self.write_u64(i as u64);
    }

    #[inline(always)]
    fn write_i8(&mut self, i: i8) {
        self.write_u8(i as u8);
    }

    #[inline(always)]
    fn write_i16(&mut self, i: i16) {
        self.write_u16(i as u16);
    }

    #[inline(always)]
    fn write_i32(&mut self, i: i32) {
        self.write_u32(i as u32);
    }

    #[inline(always)]
    fn write_i64(&mut self, i: i64) {
        self.write_u64(i as u64);
    }

    #[inline(always)]
    fn write_i128(&mut self, i: i128) {
        self.write_u128(i as u128);
    }

    #[inline(always)]
    fn write_isize(&mut self, i: isize) {
        self.write_usize(i as usize);
    }
}
