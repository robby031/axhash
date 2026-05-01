#![no_std]

use core::hash::{BuildHasher, Hash, Hasher};

const SECRET: [u64; 4] = [
    0x2d35_8dcc_aa6c_78a5,
    0x8bb8_4b93_962e_acc9,
    0x4b33_a62e_d433_d4a3,
    0x4d5a_2da5_1de1_aa47,
];

const STRIPE_SECRET: [u64; 4] = [
    0xa076_1d64_78bd_642f,
    0xe703_7ed1_a0b4_28db,
    0x8ebc_6af0_9c88_c6e3,
    0x5899_65cc_7537_4cc3,
];

#[inline(always)]
pub const fn folded_multiply(x: u64, y: u64) -> u64 {
    let product = (x as u128) * (y as u128);
    (product as u64) ^ ((product >> 64) as u64)
}

#[inline(always)]
const fn seed_lane(seed: u64, lane: usize) -> u64 {
    folded_multiply(
        seed.rotate_left((lane as u32) * 17 + 7) ^ SECRET[lane],
        STRIPE_SECRET[lane],
    )
}

#[inline(always)]
fn avalanche(mut x: u64) -> u64 {
    x ^= x >> 32;
    x = folded_multiply(x ^ SECRET[1], x.rotate_left(19) ^ STRIPE_SECRET[2]);
    x ^= x >> 29;
    x = folded_multiply(x ^ SECRET[3], x.rotate_left(27) ^ STRIPE_SECRET[0]);
    x ^ (x >> 28)
}

#[inline(always)]
unsafe fn read_u32(ptr: *const u8) -> u32 {
    unsafe { u32::from_le(core::ptr::read_unaligned(ptr.cast::<u32>())) }
}

#[inline(always)]
unsafe fn read_u64(ptr: *const u8) -> u64 {
    unsafe { u64::from_le(core::ptr::read_unaligned(ptr.cast::<u64>())) }
}

#[inline(always)]
unsafe fn tail3(ptr: *const u8, len: usize) -> u64 {
    debug_assert!(len > 0);
    let a = unsafe { *ptr } as u64;
    let b = unsafe { *ptr.add(len >> 1) } as u64;
    let c = unsafe { *ptr.add(len - 1) } as u64;
    a | (b << 24) | (c << 48)
}

#[inline(always)]
unsafe fn hash_bytes_short(ptr: *const u8, len: usize, acc: u64) -> u64 {
    debug_assert!(len <= 16);

    let (a, b) = if len >= 8 {
        let lo = unsafe { read_u64(ptr) };
        let hi = unsafe { read_u64(ptr.add(len - 8)) };
        (lo ^ SECRET[1], hi ^ STRIPE_SECRET[1])
    } else if len >= 4 {
        let lo = unsafe { read_u32(ptr) } as u64;
        let hi = unsafe { read_u32(ptr.add(len - 4)) } as u64;
        (
            lo | (hi << 32) ^ SECRET[2],
            (len as u64).rotate_left(17) ^ STRIPE_SECRET[2],
        )
    } else if len > 0 {
        (
            unsafe { tail3(ptr, len) } ^ SECRET[0],
            (len as u64).rotate_left(49) ^ STRIPE_SECRET[3],
        )
    } else {
        (SECRET[0], STRIPE_SECRET[0])
    };

    folded_multiply(a ^ acc.rotate_left(len as u32), b ^ acc)
}

#[inline(always)]
unsafe fn hash_bytes_medium(ptr: *const u8, len: usize, acc: u64) -> u64 {
    debug_assert!((17..=32).contains(&len));

    let a = unsafe { read_u64(ptr) };
    let b = unsafe { read_u64(ptr.add(8)) };
    let c = unsafe { read_u64(ptr.add(len - 16)) };
    let d = unsafe { read_u64(ptr.add(len - 8)) };

    let x = folded_multiply(a ^ SECRET[0], b ^ STRIPE_SECRET[0] ^ acc);
    let y = folded_multiply(c ^ SECRET[1], d ^ STRIPE_SECRET[1] ^ (len as u64));
    folded_multiply(
        x ^ y.rotate_left(17),
        acc ^ STRIPE_SECRET[3] ^ (len as u64).rotate_left(7),
    )
}

#[inline(always)]
unsafe fn hash_bytes_long(ptr: *const u8, len: usize, acc: u64) -> u64 {
    let mut s0 = acc ^ SECRET[0];
    let mut s1 = acc.rotate_left(17) ^ SECRET[1];
    let mut s2 = acc.rotate_left(33) ^ SECRET[2];
    let mut s3 = acc.rotate_left(49) ^ SECRET[3];

    let mut offset = 0usize;
    while offset + 64 <= len {
        let w0 = unsafe { read_u64(ptr.add(offset)) };
        let w1 = unsafe { read_u64(ptr.add(offset + 8)) };
        let w2 = unsafe { read_u64(ptr.add(offset + 16)) };
        let w3 = unsafe { read_u64(ptr.add(offset + 24)) };
        let w4 = unsafe { read_u64(ptr.add(offset + 32)) };
        let w5 = unsafe { read_u64(ptr.add(offset + 40)) };
        let w6 = unsafe { read_u64(ptr.add(offset + 48)) };
        let w7 = unsafe { read_u64(ptr.add(offset + 56)) };

        s0 = folded_multiply(s0 ^ w0, STRIPE_SECRET[0] ^ w1);
        s1 = folded_multiply(s1 ^ w2, STRIPE_SECRET[1] ^ w3);
        s2 = folded_multiply(s2 ^ w4, STRIPE_SECRET[2] ^ w5);
        s3 = folded_multiply(s3 ^ w6, STRIPE_SECRET[3] ^ w7);

        offset += 64;
    }

    if offset + 32 <= len {
        let w0 = unsafe { read_u64(ptr.add(offset)) };
        let w1 = unsafe { read_u64(ptr.add(offset + 8)) };
        let w2 = unsafe { read_u64(ptr.add(offset + 16)) };
        let w3 = unsafe { read_u64(ptr.add(offset + 24)) };

        s0 = folded_multiply(s0 ^ w0, STRIPE_SECRET[1] ^ w1);
        s1 = folded_multiply(s1 ^ w2, STRIPE_SECRET[2] ^ w3);

        offset += 32;
    }

    if offset + 16 <= len {
        let a = unsafe { read_u64(ptr.add(offset)) };
        let b = unsafe { read_u64(ptr.add(offset + 8)) };
        s2 = folded_multiply(s2 ^ a, SECRET[0] ^ b);
        s1 ^= folded_multiply(b ^ SECRET[1], a.rotate_left(19) ^ STRIPE_SECRET[3]);
        offset += 16;
    }

    let remaining = len - offset;
    if remaining > 0 {
        let tail = unsafe { hash_bytes_short(ptr.add(offset), remaining, s3 ^ len as u64) };
        s3 ^= tail;
        s0 ^= tail.rotate_left(13);
    }

    let x = folded_multiply(s0 ^ STRIPE_SECRET[0], s1 ^ STRIPE_SECRET[1]);
    let y = folded_multiply(s2 ^ STRIPE_SECRET[2], s3 ^ STRIPE_SECRET[3]);
    folded_multiply(x ^ (len as u64).rotate_left(17), y ^ acc.rotate_left(9))
}

#[inline(always)]
fn hash_bytes_core(bytes: &[u8], acc: u64) -> u64 {
    let len = bytes.len();
    let rotated = acc.rotate_right(len as u32);
    unsafe {
        if len <= 16 {
            hash_bytes_short(bytes.as_ptr(), len, rotated)
        } else if len <= 32 {
            hash_bytes_medium(bytes.as_ptr(), len, rotated)
        } else {
            hash_bytes_long(bytes.as_ptr(), len, rotated)
        }
    }
}

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

#[derive(Clone, Copy, Default)]
pub struct AxBuildHasher {
    prepared_seed: u64,
}

impl AxBuildHasher {
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

#[inline(always)]
pub fn axhash_of_seeded<T: Hash>(data: &T, seed: u64) -> u64 {
    let mut hasher = AxHasher::new_with_seed(seed);
    data.hash(&mut hasher);
    hasher.finish()
}

#[inline(always)]
pub fn axhash_seeded(bytes: &[u8], seed: u64) -> u64 {
    avalanche(hash_bytes_core(bytes, seed_lane(seed, 0)))
}

impl Hasher for AxHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        if self.sponge_bits == 0 {
            self.acc
        } else {
            let lo = self.sponge as u64;
            let hi = (self.sponge >> 64) as u64;
            folded_multiply(lo ^ self.acc, hi ^ SECRET[1])
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

#[cfg(test)]
mod tests {
    use super::{AxHasher, axhash_of_seeded, axhash_seeded};
    use core::hash::{Hash, Hasher};

    #[derive(Hash)]
    struct DemoRecord {
        id: u64,
        shard: u32,
        flags: u32,
    }

    #[test]
    fn hash_is_deterministic_for_bytes() {
        let data = b"axhash regression seed";
        let a = axhash_seeded(data, 0x1234_5678_9abc_def0);
        let b = axhash_seeded(data, 0x1234_5678_9abc_def0);
        assert_eq!(a, b);
    }

    #[test]
    fn hash_changes_when_seed_changes() {
        let data = b"same payload different seed";
        let a = axhash_seeded(data, 1);
        let b = axhash_seeded(data, 2);
        assert_ne!(a, b);
    }

    #[test]
    fn hash_trait_path_is_deterministic() {
        let record = DemoRecord {
            id: 42,
            shard: 7,
            flags: 3,
        };
        let a = axhash_of_seeded(&record, 0xdead_beef);
        let b = axhash_of_seeded(&record, 0xdead_beef);
        assert_eq!(a, b);
    }

    #[test]
    fn primitive_writes_produce_a_stable_finish() {
        let mut hasher = AxHasher::new_with_seed(0x4444);
        hasher.write_u64(0x0102_0304_0506_0708);
        hasher.write_u32(0xaabb_ccdd);
        hasher.write_u16(0xeeff);
        hasher.write_u8(0x11);
        let value = hasher.finish();
        assert_ne!(value, 0);
    }
}
