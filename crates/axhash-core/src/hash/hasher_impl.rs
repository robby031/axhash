use super::AxHasher;

use crate::bytes::hash_bytes_core;
use crate::constants::SECRET;
use crate::math::{avalanche, folded_multiply};

use core::hash::Hasher;

impl Hasher for AxHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        // Compute the post-flush accumulator without mutating self.
        // Avoids a 24-byte clone on every HashMap lookup/insert.
        let final_acc = if self.sponge_bits == 0 {
            self.acc
        } else {
            let lo = self.sponge as u64;
            let hi = (self.sponge >> 64) as u64;
            folded_multiply(lo ^ self.acc, hi ^ SECRET[1])
        };
        avalanche(final_acc)
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
        self.flush_sponge();

        self.acc = folded_multiply(self.acc ^ i as u64, SECRET[1]);
    }

    #[inline(always)]
    fn write_u64(&mut self, i: u64) {
        self.flush_sponge();

        self.acc = folded_multiply(self.acc ^ i, SECRET[1]);
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
