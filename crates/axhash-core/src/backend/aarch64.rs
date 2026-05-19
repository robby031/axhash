use crate::backend::finalize_vector;
use crate::constants::{SECRET, STRIPE_SECRET};
use crate::math::folded_multiply;
use crate::memory::r_128;
use core::arch::aarch64::*;

#[target_feature(enable = "neon")]
#[inline]
unsafe fn lane_lo(v: uint8x16_t) -> u64 {
    vgetq_lane_u64(vreinterpretq_u64_u8(v), 0)
}

#[target_feature(enable = "neon")]
#[inline]
unsafe fn lane_hi(v: uint8x16_t) -> u64 {
    vgetq_lane_u64(vreinterpretq_u64_u8(v), 1)
}

#[target_feature(enable = "neon", enable = "aes")]
#[inline]
unsafe fn process_128_block_aarch64(
    ptr: *const u8,
    v0: &mut uint8x16_t,
    v1: &mut uint8x16_t,
    v2: &mut uint8x16_t,
    v3: &mut uint8x16_t,
    v4: &mut uint8x16_t,
    v5: &mut uint8x16_t,
    v6: &mut uint8x16_t,
    v7: &mut uint8x16_t,
) {
    let d0 = unsafe { r_128(ptr) };
    let d1 = unsafe { r_128(ptr.add(16)) };
    let d2 = unsafe { r_128(ptr.add(32)) };
    let d3 = unsafe { r_128(ptr.add(48)) };
    let d4 = unsafe { r_128(ptr.add(64)) };
    let d5 = unsafe { r_128(ptr.add(80)) };
    let d6 = unsafe { r_128(ptr.add(96)) };
    let d7 = unsafe { r_128(ptr.add(112)) };

    *v1 = veorq_u8(*v1, *v0);
    *v3 = veorq_u8(*v3, *v2);
    *v5 = veorq_u8(*v5, *v4);
    *v7 = veorq_u8(*v7, *v6);

    *v0 = vaesmcq_u8(vaeseq_u8(*v0, d0));
    *v1 = vaesmcq_u8(vaeseq_u8(*v1, d1));
    *v2 = vaesmcq_u8(vaeseq_u8(*v2, d2));
    *v3 = vaesmcq_u8(vaeseq_u8(*v3, d3));
    *v4 = vaesmcq_u8(vaeseq_u8(*v4, d4));
    *v5 = vaesmcq_u8(vaeseq_u8(*v5, d5));
    *v6 = vaesmcq_u8(vaeseq_u8(*v6, d6));
    *v7 = vaesmcq_u8(vaeseq_u8(*v7, d7));
}

#[target_feature(enable = "neon", enable = "aes")]
pub(crate) unsafe fn hash_bytes_long(ptr: *const u8, len: usize, acc: u64) -> u64 {
    let mut v0 = vreinterpretq_u8_u64(vdupq_n_u64(SECRET[0] ^ acc));
    let mut v1 = vreinterpretq_u8_u64(vdupq_n_u64(STRIPE_SECRET[0] ^ acc));
    let mut v2 = vreinterpretq_u8_u64(vdupq_n_u64(SECRET[1] ^ acc));
    let mut v3 = vreinterpretq_u8_u64(vdupq_n_u64(STRIPE_SECRET[1] ^ acc));
    let mut v4 = vreinterpretq_u8_u64(vdupq_n_u64(SECRET[2] ^ acc));
    let mut v5 = vreinterpretq_u8_u64(vdupq_n_u64(STRIPE_SECRET[2] ^ acc));
    let mut v6 = vreinterpretq_u8_u64(vdupq_n_u64(SECRET[3] ^ acc));
    let mut v7 = vreinterpretq_u8_u64(vdupq_n_u64(STRIPE_SECRET[3] ^ acc));

    let mut offset = 0usize;
    while offset + 128 <= len {
        unsafe {
            process_128_block_aarch64(
                ptr.add(offset),
                &mut v0,
                &mut v1,
                &mut v2,
                &mut v3,
                &mut v4,
                &mut v5,
                &mut v6,
                &mut v7,
            );
        }
        offset += 128;
    }
    unsafe {
        process_128_block_aarch64(
            ptr.add(len - 128),
            &mut v0,
            &mut v1,
            &mut v2,
            &mut v3,
            &mut v4,
            &mut v5,
            &mut v6,
            &mut v7,
        );
    }

    let (l0, h0) = unsafe { (lane_lo(v0), lane_hi(v0)) };
    let (l1, h1) = unsafe { (lane_lo(v1), lane_hi(v1)) };
    let (l2, h2) = unsafe { (lane_lo(v2), lane_hi(v2)) };
    let (l3, h3) = unsafe { (lane_lo(v3), lane_hi(v3)) };
    let (l4, h4) = unsafe { (lane_lo(v4), lane_hi(v4)) };
    let (l5, h5) = unsafe { (lane_lo(v5), lane_hi(v5)) };
    let (l6, h6) = unsafe { (lane_lo(v6), lane_hi(v6)) };
    let (l7, h7) = unsafe { (lane_lo(v7), lane_hi(v7)) };

    let p0 = folded_multiply(l0 ^ STRIPE_SECRET[0], h1 ^ SECRET[0]);
    let p1 = folded_multiply(l2 ^ STRIPE_SECRET[1], h3 ^ SECRET[1]);
    let p2 = folded_multiply(l4 ^ STRIPE_SECRET[2], h5 ^ SECRET[2]);
    let p3 = folded_multiply(l6 ^ STRIPE_SECRET[3], h7 ^ SECRET[3]);
    let q0 = folded_multiply(h0 ^ SECRET[1], l1 ^ STRIPE_SECRET[1]);
    let q1 = folded_multiply(h2 ^ SECRET[2], l3 ^ STRIPE_SECRET[2]);
    let q2 = folded_multiply(h4 ^ SECRET[3], l5 ^ STRIPE_SECRET[3]);
    let q3 = folded_multiply(h6 ^ SECRET[0], l7 ^ STRIPE_SECRET[0]);

    let lo = folded_multiply(p0 ^ q1, p2 ^ q3);
    let hi = folded_multiply(p1 ^ q2, p3 ^ q0);

    finalize_vector(lo, hi, len)
}
