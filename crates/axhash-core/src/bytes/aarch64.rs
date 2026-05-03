use crate::bytes::finalize_vector;
use crate::constants::{SECRET, STRIPE_SECRET};
use crate::memory::r_128;
use core::arch::aarch64::*;

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
        let d0 = unsafe { r_128(ptr.add(offset)) };
        let d1 = unsafe { r_128(ptr.add(offset + 16)) };
        let d2 = unsafe { r_128(ptr.add(offset + 32)) };
        let d3 = unsafe { r_128(ptr.add(offset + 48)) };
        let d4 = unsafe { r_128(ptr.add(offset + 64)) };
        let d5 = unsafe { r_128(ptr.add(offset + 80)) };
        let d6 = unsafe { r_128(ptr.add(offset + 96)) };
        let d7 = unsafe { r_128(ptr.add(offset + 112)) };

        v0 = vaesmcq_u8(vaeseq_u8(v0, d0));
        v1 = vaesmcq_u8(vaeseq_u8(v1, d1));
        v2 = vaesmcq_u8(vaeseq_u8(v2, d2));
        v3 = vaesmcq_u8(vaeseq_u8(v3, d3));
        v4 = vaesmcq_u8(vaeseq_u8(v4, d4));
        v5 = vaesmcq_u8(vaeseq_u8(v5, d5));
        v6 = vaesmcq_u8(vaeseq_u8(v6, d6));
        v7 = vaesmcq_u8(vaeseq_u8(v7, d7));

        v1 = veorq_u8(v1, v0);
        v3 = veorq_u8(v3, v2);
        v5 = veorq_u8(v5, v4);
        v7 = veorq_u8(v7, v6);

        offset += 128;
    }

    let tail = len - 128;
    let d0 = unsafe { r_128(ptr.add(tail)) };
    let d1 = unsafe { r_128(ptr.add(tail + 16)) };
    let d2 = unsafe { r_128(ptr.add(tail + 32)) };
    let d3 = unsafe { r_128(ptr.add(tail + 48)) };
    let d4 = unsafe { r_128(ptr.add(tail + 64)) };
    let d5 = unsafe { r_128(ptr.add(tail + 80)) };
    let d6 = unsafe { r_128(ptr.add(tail + 96)) };
    let d7 = unsafe { r_128(ptr.add(tail + 112)) };

    v0 = vaesmcq_u8(vaeseq_u8(v0, d0));
    v1 = vaesmcq_u8(vaeseq_u8(v1, d1));
    v2 = vaesmcq_u8(vaeseq_u8(v2, d2));
    v3 = vaesmcq_u8(vaeseq_u8(v3, d3));
    v4 = vaesmcq_u8(vaeseq_u8(v4, d4));
    v5 = vaesmcq_u8(vaeseq_u8(v5, d5));
    v6 = vaesmcq_u8(vaeseq_u8(v6, d6));
    v7 = vaesmcq_u8(vaeseq_u8(v7, d7));

    let sum0 = veorq_u8(veorq_u8(v0, v2), veorq_u8(v4, v6));
    let sum1 = veorq_u8(veorq_u8(v1, v3), veorq_u8(v5, v7));
    let final_vec = veorq_u8(sum0, sum1);

    let final_u64x2 = vreinterpretq_u64_u8(final_vec);
    let lo = vgetq_lane_u64(final_u64x2, 0);
    let hi = vgetq_lane_u64(final_u64x2, 1);

    finalize_vector(lo, hi, len)
}
