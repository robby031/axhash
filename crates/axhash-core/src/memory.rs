#[inline(always)]
pub(crate) unsafe fn r_u32(ptr: *const u8) -> u32 {
    unsafe { u32::from_le(core::ptr::read_unaligned(ptr.cast::<u32>())) }
}

#[inline(always)]
pub(crate) unsafe fn r_u64(ptr: *const u8) -> u64 {
    unsafe { u64::from_le(core::ptr::read_unaligned(ptr.cast::<u64>())) }
}

#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::{uint8x16_t, vld1q_u8};

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
pub(crate) unsafe fn r_128(ptr: *const u8) -> uint8x16_t {
    unsafe { vld1q_u8(ptr) }
}
