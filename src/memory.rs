#[inline(always)]
pub(crate) unsafe fn read_u32(ptr: *const u8) -> u32 {
    unsafe { u32::from_le(core::ptr::read_unaligned(ptr.cast::<u32>())) }
}

#[inline(always)]
pub(crate) unsafe fn read_u64(ptr: *const u8) -> u64 {
    unsafe { u64::from_le(core::ptr::read_unaligned(ptr.cast::<u64>())) }
}
