use crate::constants::{SECRET, STRIPE_SECRET};
use crate::math::folded_multiply;
use crate::memory::r_u64;

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
