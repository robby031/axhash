#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
use crate::constants::FINAL_MIX;
use crate::constants::{SECRET, STRIPE_SECRET};
use crate::math::folded_multiply;
use crate::memory::r_u64;

mod scalar;

#[cfg(target_arch = "aarch64")]
mod aarch64;

#[cfg(target_arch = "x86_64")]
mod x86_64;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Backend {
    Scalar,
    #[cfg(target_arch = "aarch64")]
    Aarch64AesNeon,
    #[cfg(target_arch = "x86_64")]
    X86_64AesAvx2,
}

#[inline(always)]
unsafe fn read_partial_u64(ptr: *const u8, len: usize) -> u64 {
    debug_assert!((1..=7).contains(&len));
    if len >= 4 {
        let a = u32::from_le(unsafe { core::ptr::read_unaligned(ptr.cast::<u32>()) });
        let b = u32::from_le(unsafe { core::ptr::read_unaligned(ptr.add(len - 4).cast::<u32>()) });
        (a as u64) | ((b as u64) << 32)
    } else {
        let b0 = unsafe { *ptr } as u64;
        let b1 = if len > 1 {
            (unsafe { *ptr.add(1) }) as u64
        } else {
            0
        };
        let b2 = if len > 2 {
            (unsafe { *ptr.add(2) }) as u64
        } else {
            0
        };
        b0 | (b1 << 8) | (b2 << 16)
    }
}

#[inline(always)]
pub(crate) unsafe fn hash_bytes_short(ptr: *const u8, len: usize, acc: u64) -> u64 {
    debug_assert!((1..=16).contains(&len));

    let (lo, hi) = if len >= 8 {
        let lo = unsafe { r_u64(ptr) };
        let hi = unsafe { r_u64(ptr.add(len - 8)) };
        (lo, hi)
    } else {
        let lo = unsafe { read_partial_u64(ptr, len) };
        (lo, lo)
    };

    let mut a = lo ^ SECRET[1];
    let mut b = hi ^ STRIPE_SECRET[1];

    a ^= len as u64;
    b ^= acc;

    let m1 = a.wrapping_mul(0x9E3779B185EBCA87);
    let m2 = b.wrapping_mul(0xC2B2AE3D27D4EB4F);

    let h = m1 ^ m2;

    h ^ (h >> 32)
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

#[inline(always)]
fn hash_bytes_long(ptr: *const u8, len: usize, acc: u64) -> u64 {
    match selected_backend() {
        #[cfg(target_arch = "aarch64")]
        Backend::Aarch64AesNeon => {
            // SAFETY: selected_backend only returns this variant when CPU support is present.
            unsafe { aarch64::hash_bytes_long(ptr, len, acc) }
        }
        #[cfg(target_arch = "x86_64")]
        Backend::X86_64AesAvx2 => {
            // SAFETY: selected_backend only returns this variant when CPU support is present.
            unsafe { x86_64::hash_bytes_long(ptr, len, acc) }
        }
        Backend::Scalar => {
            // SAFETY: scalar path only reads within the provided byte slice bounds.
            unsafe { scalar::hash_bytes_long(ptr, len, acc) }
        }
    }
}
#[inline(always)]
pub(crate) fn hash_bytes_core(bytes: &[u8], acc: u64) -> u64 {
    let len = bytes.len();

    if len == 0 {
        return acc;
    }

    unsafe {
        if len <= 16 {
            hash_bytes_short(bytes.as_ptr(), len, acc)
        } else if len <= 32 {
            let rotated = acc.rotate_right(len as u32);
            hash_bytes_17_32(bytes.as_ptr(), len, rotated)
        } else if len <= 64 {
            let rotated = acc.rotate_right(len as u32);
            hash_bytes_33_64(bytes.as_ptr(), len, rotated)
        } else if len <= 128 {
            let rotated = acc.rotate_right(len as u32);
            hash_bytes_65_128(bytes.as_ptr(), len, rotated)
        } else {
            let rotated = acc.rotate_right(len as u32);
            hash_bytes_long(bytes.as_ptr(), len, rotated)
        }
    }
}

#[cfg(all(target_arch = "aarch64", feature = "std"))]
#[inline(always)]
pub(crate) fn selected_backend() -> Backend {
    if std::arch::is_aarch64_feature_detected!("aes")
        && std::arch::is_aarch64_feature_detected!("neon")
    {
        Backend::Aarch64AesNeon
    } else {
        Backend::Scalar
    }
}

#[cfg(all(
    target_arch = "aarch64",
    not(feature = "std"),
    target_feature = "aes",
    target_feature = "neon"
))]
#[inline(always)]
pub(crate) fn selected_backend() -> Backend {
    Backend::Aarch64AesNeon
}

#[cfg(all(
    target_arch = "aarch64",
    not(feature = "std"),
    not(all(target_feature = "aes", target_feature = "neon"))
))]
#[inline(always)]
pub(crate) fn selected_backend() -> Backend {
    Backend::Scalar
}

#[cfg(all(target_arch = "x86_64", feature = "std"))]
#[inline(always)]
pub(crate) fn selected_backend() -> Backend {
    if std::arch::is_x86_feature_detected!("aes") && std::arch::is_x86_feature_detected!("avx2") {
        Backend::X86_64AesAvx2
    } else {
        Backend::Scalar
    }
}

#[cfg(all(
    target_arch = "x86_64",
    not(feature = "std"),
    target_feature = "aes",
    target_feature = "avx2"
))]
#[inline(always)]
pub(crate) fn selected_backend() -> Backend {
    Backend::X86_64AesAvx2
}

#[cfg(all(
    target_arch = "x86_64",
    not(feature = "std"),
    not(all(target_feature = "aes", target_feature = "avx2"))
))]
#[inline(always)]
pub(crate) fn selected_backend() -> Backend {
    Backend::Scalar
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
#[inline(always)]
pub(crate) fn selected_backend() -> Backend {
    Backend::Scalar
}

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[inline(always)]
pub(super) fn finalize_vector(lo: u64, hi: u64, len: usize) -> u64 {
    folded_multiply(lo ^ FINAL_MIX, hi ^ (len as u64))
}
