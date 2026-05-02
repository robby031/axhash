use crate::constants::{SECRET, STRIPE_SECRET};
use crate::math::{avalanche, folded_multiply};
use crate::memory::{read_u32, read_u64};

#[inline(always)]
pub(crate) unsafe fn hash_bytes_short(ptr: *const u8, len: usize, acc: u64) -> u64 {
    let h = if len >= 8 {
        let a = unsafe { read_u64(ptr) } ^ SECRET[1] ^ (len as u64);
        let b = unsafe { read_u64(ptr.add(len - 8)) } ^ STRIPE_SECRET[1] ^ acc;
        folded_multiply(a, b)
    } else if len >= 4 {
        let a = (unsafe { read_u32(ptr) } as u64) ^ SECRET[2] ^ (len as u64);
        let b = (unsafe { read_u32(ptr.add(len - 4)) } as u64) ^ STRIPE_SECRET[2] ^ acc;
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
pub(crate) unsafe fn hash_bytes_medium(ptr: *const u8, len: usize, acc: u64) -> u64 {
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
pub(crate) unsafe fn hash_bytes_long(ptr: *const u8, len: usize, acc: u64) -> u64 {
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
pub(crate) fn hash_bytes_core(bytes: &[u8], acc: u64) -> u64 {
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
