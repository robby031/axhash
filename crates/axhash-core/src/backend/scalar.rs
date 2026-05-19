use crate::constants::{SECRET, STRIPE_SECRET};
use crate::math::folded_multiply;
use crate::memory::r_u64;

#[inline(always)]
pub(super) unsafe fn read_partial_u64(ptr: *const u8, len: usize) -> u64 {
    debug_assert!((1..=7).contains(&len));
    if len >= 4 {
        let a = u32::from_le(unsafe { core::ptr::read_unaligned(ptr.cast::<u32>()) });
        let b = u32::from_le(unsafe { core::ptr::read_unaligned(ptr.add(len - 4).cast::<u32>()) });
        (a as u64) | ((b as u64) << 32)
    } else {
        let c1 = unsafe { *ptr } as u64;
        let c2 = unsafe { *ptr.add(len >> 1) } as u64;
        let c3 = unsafe { *ptr.add(len - 1) } as u64;
        let raw = c1 | (c2 << 8) | (c3 << 16);
        let mask = (1u64 << (len << 3)) - 1;
        raw & mask
    }
}

#[inline(always)]
pub(super) unsafe fn hash_bytes_short(ptr: *const u8, len: usize, acc: u64) -> u64 {
    debug_assert!((1..=16).contains(&len));
    let len_u64 = len as u64;
    let len_mix = len_u64.wrapping_mul(0x9E3779B97F4A7C15);

    if len >= 8 {
        let lo = unsafe { r_u64(ptr) };
        let hi = unsafe { r_u64(ptr.add(len - 8)) };

        let m0 = folded_multiply(lo ^ SECRET[0], hi ^ STRIPE_SECRET[0] ^ len_mix);
        folded_multiply(m0 ^ acc, len_mix ^ SECRET[1])
    } else if len == 4 {
        let v = u32::from_le(unsafe { core::ptr::read_unaligned(ptr.cast::<u32>()) }) as u64;
        let lo = v | (v << 32);
        let hi = lo.rotate_left(17) ^ 4 ^ SECRET[0];
        let m0 = folded_multiply(lo ^ SECRET[1] ^ acc, hi ^ STRIPE_SECRET[1]);
        folded_multiply(m0, len_mix ^ STRIPE_SECRET[0])
    } else {
        let lo = unsafe { read_partial_u64(ptr, len) };
        let hi = lo.rotate_left(17) ^ len_u64 ^ SECRET[0];

        let m0 = folded_multiply(lo ^ SECRET[1] ^ acc, hi ^ STRIPE_SECRET[1]);
        folded_multiply(m0, len_mix ^ STRIPE_SECRET[0])
    }
}

#[inline(always)]
pub(super) unsafe fn hash_bytes_17_32(ptr: *const u8, len: usize, acc: u64) -> u64 {
    let a = unsafe { r_u64(ptr) };
    let b = unsafe { r_u64(ptr.add(8)) };
    let c = unsafe { r_u64(ptr.add(len - 16)) };
    let d = unsafe { r_u64(ptr.add(len - 8)) };

    let len_mix = (len as u64).wrapping_mul(0x9E3779B97F4A7C15);

    let front = folded_multiply(a ^ SECRET[0], b ^ STRIPE_SECRET[0]);
    let back = folded_multiply(c ^ SECRET[1], d ^ STRIPE_SECRET[1] ^ len_mix);
    let mix = front ^ back.rotate_left(17);

    folded_multiply(mix ^ acc, len_mix ^ SECRET[2])
}

#[inline(always)]
pub(super) unsafe fn hash_bytes_33_64(ptr: *const u8, len: usize, acc: u64) -> u64 {
    let w0 = unsafe { r_u64(ptr) };
    let w1 = unsafe { r_u64(ptr.add(8)) };
    let w2 = unsafe { r_u64(ptr.add(16)) };
    let w3 = unsafe { r_u64(ptr.add(24)) };
    let w4 = unsafe { r_u64(ptr.add(len - 32)) };
    let w5 = unsafe { r_u64(ptr.add(len - 24)) };
    let w6 = unsafe { r_u64(ptr.add(len - 16)) };
    let w7 = unsafe { r_u64(ptr.add(len - 8)) };

    let len_mix = (len as u64).wrapping_mul(0x9E3779B97F4A7C15);

    let f0 = folded_multiply(w0 ^ SECRET[0], w1 ^ STRIPE_SECRET[0]);
    let f1 = folded_multiply(w2 ^ SECRET[1], w3 ^ STRIPE_SECRET[1]);
    let front = f0 ^ f1.rotate_left(17);

    let b0 = folded_multiply(w4 ^ SECRET[2], w5 ^ STRIPE_SECRET[2] ^ len_mix);
    let b1 = folded_multiply(w6 ^ SECRET[3], w7 ^ STRIPE_SECRET[3]);
    let back = b0 ^ b1.rotate_left(21);

    let m = folded_multiply(front ^ acc, back ^ STRIPE_SECRET[0]);
    folded_multiply(m ^ len_mix.rotate_left(23), SECRET[1] ^ acc)
}

#[inline(always)]
pub(super) unsafe fn hash_bytes_65_128(ptr: *const u8, len: usize, acc: u64) -> u64 {
    let w0 = unsafe { r_u64(ptr) };
    let w1 = unsafe { r_u64(ptr.add(8)) };
    let w2 = unsafe { r_u64(ptr.add(16)) };
    let w3 = unsafe { r_u64(ptr.add(24)) };
    let w4 = unsafe { r_u64(ptr.add(32)) };
    let w5 = unsafe { r_u64(ptr.add(40)) };
    let w6 = unsafe { r_u64(ptr.add(48)) };
    let w7 = unsafe { r_u64(ptr.add(56)) };
    let w8 = unsafe { r_u64(ptr.add(len - 64)) };
    let w9 = unsafe { r_u64(ptr.add(len - 56)) };
    let wa = unsafe { r_u64(ptr.add(len - 48)) };
    let wb = unsafe { r_u64(ptr.add(len - 40)) };
    let wc = unsafe { r_u64(ptr.add(len - 32)) };
    let wd = unsafe { r_u64(ptr.add(len - 24)) };
    let we = unsafe { r_u64(ptr.add(len - 16)) };
    let wf = unsafe { r_u64(ptr.add(len - 8)) };
    let m0 = folded_multiply(w0 ^ SECRET[0], w1 ^ STRIPE_SECRET[0] ^ acc);
    let m1 = folded_multiply(w2 ^ SECRET[1], w3 ^ STRIPE_SECRET[1]);
    let m2 = folded_multiply(w4 ^ SECRET[2], w5 ^ STRIPE_SECRET[2]);
    let m3 = folded_multiply(w6 ^ SECRET[3], w7 ^ STRIPE_SECRET[3]);
    let len_mix = (len as u64).wrapping_mul(0x9E3779B97F4A7C15);
    let m4 = folded_multiply(w8 ^ SECRET[0], w9 ^ STRIPE_SECRET[0] ^ len_mix);
    let m5 = folded_multiply(wa ^ SECRET[1], wb ^ STRIPE_SECRET[1]);
    let m6 = folded_multiply(wc ^ SECRET[2], wd ^ STRIPE_SECRET[2]);
    let m7 = folded_multiply(we ^ SECRET[3], wf ^ STRIPE_SECRET[3]);
    let s0 = m0 ^ m4.rotate_left(17);
    let s1 = m1 ^ m5.rotate_left(19);
    let s2 = m2 ^ m6.rotate_left(23);
    let s3 = m3 ^ m7.rotate_left(29);
    let x = folded_multiply(s0 ^ STRIPE_SECRET[0], s1 ^ STRIPE_SECRET[1]);
    let y = folded_multiply(s2 ^ STRIPE_SECRET[2], s3 ^ STRIPE_SECRET[3]);
    folded_multiply(x ^ y.rotate_left(17), acc ^ len_mix)
}

#[inline(always)]
unsafe fn mix_128_block(ptr: *const u8, s0: &mut u64, s1: &mut u64, s2: &mut u64, s3: &mut u64) {
    let w0 = unsafe { r_u64(ptr) };
    let w1 = unsafe { r_u64(ptr.add(8)) };
    let w2 = unsafe { r_u64(ptr.add(16)) };
    let w3 = unsafe { r_u64(ptr.add(24)) };
    let w4 = unsafe { r_u64(ptr.add(32)) };
    let w5 = unsafe { r_u64(ptr.add(40)) };
    let w6 = unsafe { r_u64(ptr.add(48)) };
    let w7 = unsafe { r_u64(ptr.add(56)) };
    let w8 = unsafe { r_u64(ptr.add(64)) };
    let w9 = unsafe { r_u64(ptr.add(72)) };
    let wa = unsafe { r_u64(ptr.add(80)) };
    let wb = unsafe { r_u64(ptr.add(88)) };
    let wc = unsafe { r_u64(ptr.add(96)) };
    let wd = unsafe { r_u64(ptr.add(104)) };
    let we = unsafe { r_u64(ptr.add(112)) };
    let wf = unsafe { r_u64(ptr.add(120)) };

    let m0 = folded_multiply(w0 ^ SECRET[0], w1 ^ STRIPE_SECRET[0]);
    let m1 = folded_multiply(w2 ^ SECRET[1], w3 ^ STRIPE_SECRET[1]);
    let m2 = folded_multiply(w4 ^ SECRET[2], w5 ^ STRIPE_SECRET[2]);
    let m3 = folded_multiply(w6 ^ SECRET[3], w7 ^ STRIPE_SECRET[3]);

    let m4 = folded_multiply(w8 ^ SECRET[0], w9 ^ STRIPE_SECRET[0]);
    let m5 = folded_multiply(wa ^ SECRET[1], wb ^ STRIPE_SECRET[1]);
    let m6 = folded_multiply(wc ^ SECRET[2], wd ^ STRIPE_SECRET[2]);
    let m7 = folded_multiply(we ^ SECRET[3], wf ^ STRIPE_SECRET[3]);

    *s0 = s0.rotate_left(19) ^ m0 ^ m4.rotate_left(23);
    *s1 = s1.rotate_left(19) ^ m1 ^ m5.rotate_left(23);
    *s2 = s2.rotate_left(19) ^ m2 ^ m6.rotate_left(23);
    *s3 = s3.rotate_left(19) ^ m3 ^ m7.rotate_left(23);
}

#[inline(always)]
pub(crate) unsafe fn hash_bytes_long(ptr: *const u8, len: usize, acc: u64) -> u64 {
    let mut s0 = acc ^ SECRET[0];
    let mut s1 = acc.rotate_left(17) ^ SECRET[1];
    let mut s2 = acc.rotate_left(33) ^ SECRET[2];
    let mut s3 = acc.rotate_left(49) ^ SECRET[3];

    let mut offset = 0usize;
    while offset + 128 <= len {
        unsafe { mix_128_block(ptr.add(offset), &mut s0, &mut s1, &mut s2, &mut s3) };
        offset += 128;
    }
    unsafe { mix_128_block(ptr.add(len - 128), &mut s0, &mut s1, &mut s2, &mut s3) };

    let x = folded_multiply(s0 ^ STRIPE_SECRET[0], s1 ^ STRIPE_SECRET[1]);
    let y = folded_multiply(s2 ^ STRIPE_SECRET[2], s3 ^ STRIPE_SECRET[3]);

    let len_mix = (len as u64).wrapping_mul(0x9E3779B97F4A7C15);
    folded_multiply(x ^ len_mix.rotate_left(17), y ^ acc.rotate_left(9))
}
