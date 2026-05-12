#[inline(always)]
pub(crate) unsafe fn r_u64(ptr: *const u8) -> u64 {
    // SAFETY: The caller guarantees `ptr` points to at least 8 valid bytes.
    // `read_unaligned` is safe for any alignment.
    unsafe { u64::from_le(core::ptr::read_unaligned(ptr.cast::<u64>())) }
}

#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::{uint8x16_t, vld1q_u8};

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub(crate) unsafe fn r_128(ptr: *const u8) -> uint8x16_t {
    // SAFETY: The caller guarantees `ptr` points to at least 16 valid bytes.
    // `vld1q_u8` accepts unaligned pointers on AArch64.
    unsafe { vld1q_u8(ptr) }
}
