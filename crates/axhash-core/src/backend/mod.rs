#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
use crate::constants::FINAL_MIX;
use crate::math::folded_multiply;

pub(crate) mod scalar;

use scalar::{hash_bytes_17_32, hash_bytes_33_64, hash_bytes_65_128, hash_bytes_short};

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
