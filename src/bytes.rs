use crate::constants::{FINAL_MIX, SECRET, STRIPE_SECRET};
use crate::math::{avalanche, folded_multiply};
use crate::memory::r_128;
use crate::memory::{r_u32, r_u64};

#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

#[inline(always)]
pub(crate) unsafe fn hash_bytes_short(ptr: *const u8, len: usize, acc: u64) -> u64 {
    let h = if len >= 8 {
        let a = unsafe { r_u64(ptr) } ^ SECRET[1] ^ (len as u64);
        let b = unsafe { r_u64(ptr.add(len - 8)) } ^ STRIPE_SECRET[1] ^ acc;
        folded_multiply(a, b)
    } else if len >= 4 {
        let a = (unsafe { r_u32(ptr) } as u64) ^ SECRET[2] ^ (len as u64);
        let b = (unsafe { r_u32(ptr.add(len - 4)) } as u64) ^ STRIPE_SECRET[2] ^ acc;
        folded_multiply(a, b)
    } else if len > 0 {
        let a = unsafe { *ptr } as u64;
        let b = unsafe { *ptr.add(len >> 1) } as u64;
        let c = unsafe { *ptr.add(len - 1) } as u64;
        let val = (a << 16) | (b << 8) | c;
        folded_multiply(val ^ SECRET[0] ^ (len as u64), STRIPE_SECRET[3] ^ acc)
    } else {
        folded_multiply(acc ^ SECRET[0], STRIPE_SECRET[0])
    };
    avalanche(h)
}

#[inline(always)]
unsafe fn hash_bytes_17_32(ptr: *const u8, len: usize, acc: u64) -> u64 {
    let a = unsafe { r_u64(ptr) };
    let b = unsafe { r_u64(ptr.add(8)) };
    let c = unsafe { r_u64(ptr.add(len - 16)) };
    let d = unsafe { r_u64(ptr.add(len - 8)) };

    let x = folded_multiply(a ^ SECRET[0], b ^ STRIPE_SECRET[0] ^ acc);
    let y = folded_multiply(c ^ SECRET[1], d ^ STRIPE_SECRET[1] ^ (len as u64));
    folded_multiply(x ^ y.rotate_left(17), acc ^ (len as u64))
}

#[inline(always)]
unsafe fn hash_bytes_33_64(ptr: *const u8, len: usize, acc: u64) -> u64 {
    let w0 = unsafe { r_u64(ptr) };
    let w1 = unsafe { r_u64(ptr.add(8)) };
    let w2 = unsafe { r_u64(ptr.add(16)) };
    let w3 = unsafe { r_u64(ptr.add(24)) };

    let w4 = unsafe { r_u64(ptr.add(len - 32)) };
    let w5 = unsafe { r_u64(ptr.add(len - 24)) };
    let w6 = unsafe { r_u64(ptr.add(len - 16)) };
    let w7 = unsafe { r_u64(ptr.add(len - 8)) };

    let m0 = folded_multiply(w0 ^ SECRET[0], w1 ^ STRIPE_SECRET[0] ^ acc);
    let m1 = folded_multiply(w2 ^ SECRET[1], w3 ^ STRIPE_SECRET[1]);
    let m2 = folded_multiply(w4 ^ SECRET[2], w5 ^ STRIPE_SECRET[2] ^ (len as u64));
    let m3 = folded_multiply(w6 ^ SECRET[3], w7 ^ STRIPE_SECRET[3]);

    let x = folded_multiply(m0 ^ m1.rotate_left(17), STRIPE_SECRET[0] ^ acc);
    let y = folded_multiply(m2 ^ m3.rotate_left(17), STRIPE_SECRET[1] ^ (len as u64));

    folded_multiply(x ^ y.rotate_left(19), acc ^ (len as u64))
}

#[inline(always)]
unsafe fn hash_bytes_65_128(ptr: *const u8, len: usize, acc: u64) -> u64 {
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

    let m4 = folded_multiply(w8 ^ SECRET[0], w9 ^ STRIPE_SECRET[0] ^ (len as u64));
    let m5 = folded_multiply(wa ^ SECRET[1], wb ^ STRIPE_SECRET[1]);
    let m6 = folded_multiply(wc ^ SECRET[2], wd ^ STRIPE_SECRET[2]);
    let m7 = folded_multiply(we ^ SECRET[3], wf ^ STRIPE_SECRET[3]);

    let s0 = m0 ^ m4.rotate_left(17);
    let s1 = m1 ^ m5.rotate_left(19);
    let s2 = m2 ^ m6.rotate_left(23);
    let s3 = m3 ^ m7.rotate_left(29);

    let x = folded_multiply(s0 ^ STRIPE_SECRET[0], s1 ^ STRIPE_SECRET[1]);
    let y = folded_multiply(s2 ^ STRIPE_SECRET[2], s3 ^ STRIPE_SECRET[3]);

    folded_multiply(x ^ y.rotate_left(17), acc ^ (len as u64))
}

#[cfg(all(
    target_arch = "aarch64",
    target_feature = "aes",
    target_feature = "neon"
))]
#[target_feature(enable = "neon", enable = "aes")]
pub(crate) unsafe fn hash_bytes_long(ptr: *const u8, len: usize, acc: u64) -> u64 {
    unsafe {
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
            let d0 = r_128(ptr.add(offset));
            let d1 = r_128(ptr.add(offset + 16));
            let d2 = r_128(ptr.add(offset + 32));
            let d3 = r_128(ptr.add(offset + 48));
            let d4 = r_128(ptr.add(offset + 64));
            let d5 = r_128(ptr.add(offset + 80));
            let d6 = r_128(ptr.add(offset + 96));
            let d7 = r_128(ptr.add(offset + 112));

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
        let d0 = r_128(ptr.add(tail));
        let d1 = r_128(ptr.add(tail + 16));
        let d2 = r_128(ptr.add(tail + 32));
        let d3 = r_128(ptr.add(tail + 48));
        let d4 = r_128(ptr.add(tail + 64));
        let d5 = r_128(ptr.add(tail + 80));
        let d6 = r_128(ptr.add(tail + 96));
        let d7 = r_128(ptr.add(tail + 112));

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

        folded_multiply(lo ^ FINAL_MIX, hi ^ (len as u64))
    }
}

#[cfg(not(all(target_arch = "aarch64", target_feature = "aes")))]
#[inline(always)]
pub(crate) unsafe fn hash_bytes_long(ptr: *const u8, len: usize, acc: u64) -> u64 {
    let mut s0 = acc ^ SECRET[0];
    let mut s1 = acc.rotate_left(17) ^ SECRET[1];
    let mut s2 = acc.rotate_left(33) ^ SECRET[2];
    let mut s3 = acc.rotate_left(49) ^ SECRET[3];

    let mut offset = 0usize;

    while offset + 128 <= len {
        let w0 = unsafe { r_u64(ptr.add(offset)) };
        let w1 = unsafe { r_u64(ptr.add(offset + 8)) };
        let w2 = unsafe { r_u64(ptr.add(offset + 16)) };
        let w3 = unsafe { r_u64(ptr.add(offset + 24)) };
        let w4 = unsafe { r_u64(ptr.add(offset + 32)) };
        let w5 = unsafe { r_u64(ptr.add(offset + 40)) };
        let w6 = unsafe { r_u64(ptr.add(offset + 48)) };
        let w7 = unsafe { r_u64(ptr.add(offset + 56)) };

        let w8 = unsafe { r_u64(ptr.add(offset + 64)) };
        let w9 = unsafe { r_u64(ptr.add(offset + 72)) };
        let wa = unsafe { r_u64(ptr.add(offset + 80)) };
        let wb = unsafe { r_u64(ptr.add(offset + 88)) };
        let wc = unsafe { r_u64(ptr.add(offset + 96)) };
        let wd = unsafe { r_u64(ptr.add(offset + 104)) };
        let we = unsafe { r_u64(ptr.add(offset + 112)) };
        let wf = unsafe { r_u64(ptr.add(offset + 120)) };

        let m0 = folded_multiply(w0 ^ SECRET[0], w1 ^ STRIPE_SECRET[0]);
        let m1 = folded_multiply(w2 ^ SECRET[1], w3 ^ STRIPE_SECRET[1]);
        let m2 = folded_multiply(w4 ^ SECRET[2], w5 ^ STRIPE_SECRET[2]);
        let m3 = folded_multiply(w6 ^ SECRET[3], w7 ^ STRIPE_SECRET[3]);

        let m4 = folded_multiply(w8 ^ SECRET[0], w9 ^ STRIPE_SECRET[0]);
        let m5 = folded_multiply(wa ^ SECRET[1], wb ^ STRIPE_SECRET[1]);
        let m6 = folded_multiply(wc ^ SECRET[2], wd ^ STRIPE_SECRET[2]);
        let m7 = folded_multiply(we ^ SECRET[3], wf ^ STRIPE_SECRET[3]);

        s0 = s0.rotate_left(19) ^ m0 ^ m4.rotate_left(23);
        s1 = s1.rotate_left(19) ^ m1 ^ m5.rotate_left(23);
        s2 = s2.rotate_left(19) ^ m2 ^ m6.rotate_left(23);
        s3 = s3.rotate_left(19) ^ m3 ^ m7.rotate_left(23);

        offset += 128;
    }

    let tail = len - 128;
    let w0 = unsafe { r_u64(ptr.add(tail)) };
    let w1 = unsafe { r_u64(ptr.add(tail + 8)) };
    let w2 = unsafe { r_u64(ptr.add(tail + 16)) };
    let w3 = unsafe { r_u64(ptr.add(tail + 24)) };
    let w4 = unsafe { r_u64(ptr.add(tail + 32)) };
    let w5 = unsafe { r_u64(ptr.add(tail + 40)) };
    let w6 = unsafe { r_u64(ptr.add(tail + 48)) };
    let w7 = unsafe { r_u64(ptr.add(tail + 56)) };
    let w8 = unsafe { r_u64(ptr.add(tail + 64)) };
    let w9 = unsafe { r_u64(ptr.add(tail + 72)) };
    let wa = unsafe { r_u64(ptr.add(tail + 80)) };
    let wb = unsafe { r_u64(ptr.add(tail + 88)) };
    let wc = unsafe { r_u64(ptr.add(tail + 96)) };
    let wd = unsafe { r_u64(ptr.add(tail + 104)) };
    let we = unsafe { r_u64(ptr.add(tail + 112)) };
    let wf = unsafe { r_u64(ptr.add(tail + 120)) };

    let m0 = folded_multiply(w0 ^ SECRET[0], w1 ^ STRIPE_SECRET[0]);
    let m1 = folded_multiply(w2 ^ SECRET[1], w3 ^ STRIPE_SECRET[1]);
    let m2 = folded_multiply(w4 ^ SECRET[2], w5 ^ STRIPE_SECRET[2]);
    let m3 = folded_multiply(w6 ^ SECRET[3], w7 ^ STRIPE_SECRET[3]);

    let m4 = folded_multiply(w8 ^ SECRET[0], w9 ^ STRIPE_SECRET[0]);
    let m5 = folded_multiply(wa ^ SECRET[1], wb ^ STRIPE_SECRET[1]);
    let m6 = folded_multiply(wc ^ SECRET[2], wd ^ STRIPE_SECRET[2]);
    let m7 = folded_multiply(we ^ SECRET[3], wf ^ STRIPE_SECRET[3]);

    s0 = s0.rotate_left(19) ^ m0 ^ m4.rotate_left(23);
    s1 = s1.rotate_left(19) ^ m1 ^ m5.rotate_left(23);
    s2 = s2.rotate_left(19) ^ m2 ^ m6.rotate_left(23);
    s3 = s3.rotate_left(19) ^ m3 ^ m7.rotate_left(23);

    let x = folded_multiply(s0 ^ STRIPE_SECRET[0], s1 ^ STRIPE_SECRET[1]);
    let y = folded_multiply(s2 ^ STRIPE_SECRET[2], s3 ^ STRIPE_SECRET[3]);

    folded_multiply(x ^ (len as u64).rotate_left(17), y ^ acc.rotate_left(9))
}

#[inline(always)]
pub(crate) fn hash_bytes_core(bytes: &[u8], acc: u64) -> u64 {
    let len = bytes.len();
    let rotated = acc.rotate_right(len as u32);
    unsafe {
        if len <= 16 {
            hash_bytes_short(bytes.as_ptr(), len, rotated)
        } else if len <= 32 {
            hash_bytes_17_32(bytes.as_ptr(), len, rotated)
        } else if len <= 64 {
            hash_bytes_33_64(bytes.as_ptr(), len, rotated)
        } else if len <= 128 {
            hash_bytes_65_128(bytes.as_ptr(), len, rotated)
        } else {
            hash_bytes_long(bytes.as_ptr(), len, rotated)
        }
    }
}
