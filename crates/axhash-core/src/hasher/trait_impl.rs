use super::AxHasher;

use crate::backend::hash_bytes_core;
use crate::constants::SECRET;
use crate::math::{avalanche, folded_multiply};

use core::hash::Hasher;

impl Hasher for AxHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        let mut acc = self.acc;
        if self.sponge_bits != 0 {
            let lo = self.sponge as u64;
            let hi = (self.sponge >> 64) as u64;
            acc = folded_multiply(lo ^ acc, hi ^ SECRET[1]);
        }
        avalanche(acc)
    }

    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        let len = bytes.len();
        if len == 0 {
            return;
        }

        // Slice sangat pendek (<8 byte) di-buffer ke sponge — overhead
        // push_bytes lebih murah dari hash_bytes_short dispatch untuk
        // sub-word writes (write_u8 trait, partial byte streams).
        //
        // Slice 8..=16 byte tetap lewat hash_bytes_core karena hash_bytes_short
        // branch len≥8 hanya 2 folded_multiply — sudah optimal dan tidak
        // perlu sponge overhead.
        if len < 8 {
            let used = (self.sponge_bits >> 3) as usize;
            let free = 16 - used;

            if len <= free {
                self.push_bytes(bytes);
            } else {
                self.push_bytes(&bytes[..free]);
                self.flush_sponge_hot();
                self.push_bytes(&bytes[free..]);
            }
            return;
        }

        // Slice panjang: flush sponge dulu, lalu lewat hash_bytes_core
        // (AES path untuk len > 128, scalar untuk 17..=128).
        if self.sponge_bits != 0 {
            self.flush_sponge_slow();
        }
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
        if self.sponge_bits != 0 {
            self.flush_sponge_slow();
        }
        self.acc = folded_multiply(self.acc ^ i as u64, SECRET[1]);
    }

    #[inline(always)]
    fn write_u64(&mut self, i: u64) {
        if self.sponge_bits != 0 {
            self.flush_sponge_slow();
        }
        self.acc = folded_multiply(self.acc ^ i, SECRET[1]);
    }

    #[inline(always)]
    fn write_u128(&mut self, i: u128) {
        if self.sponge_bits != 0 {
            self.flush_sponge_slow();
        }
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
