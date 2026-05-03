use axhash_core::{
    AxHasher, RuntimeBackend, axhash, axhash_seeded, runtime_backend, runtime_has_aes,
};
use core::ffi::c_char;
use core::hash::Hasher;
use core::ptr::NonNull;

extern crate alloc;
use alloc::boxed::Box;

#[repr(C)]
pub struct AxHashState {
    _private: [u8; 0],
}

#[repr(C)]
pub struct AxHashIovec {
    pub ptr: *const u8,
    pub len: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxHashRuntimeBackend {
    Scalar = 0,
    Aarch64AesNeon = 1,
    X86_64AesAvx2 = 2,
}

const MAX_BATCH: usize = 1 << 16;

#[inline(always)]
fn as_state_ptr(hasher: AxHasher) -> *mut AxHashState {
    Box::into_raw(Box::new(hasher)).cast::<AxHashState>()
}

#[inline(always)]
unsafe fn state_mut<'a>(state: *mut AxHashState) -> Option<&'a mut AxHasher> {
    NonNull::new(state.cast::<AxHasher>()).map(|p| unsafe { &mut *p.as_ptr() })
}

#[inline(always)]
unsafe fn ffi_bytes_unchecked<'a>(bytes: *const u8, len: usize) -> &'a [u8] {
    unsafe { core::slice::from_raw_parts(bytes, len) }
}

#[inline(always)]
fn is_invalid_input(ptr: *const u8, len: usize) -> bool {
    (len != 0) & ptr.is_null()
}

#[inline(always)]
fn map_backend(backend: RuntimeBackend) -> AxHashRuntimeBackend {
    match backend {
        RuntimeBackend::Scalar => AxHashRuntimeBackend::Scalar,
        RuntimeBackend::Aarch64AesNeon => AxHashRuntimeBackend::Aarch64AesNeon,
        RuntimeBackend::X86_64AesAvx2 => AxHashRuntimeBackend::X86_64AesAvx2,
    }
}

#[cold]
fn fail_u64() -> u64 {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_ffi_version() -> *const c_char {
    static VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    VERSION.as_ptr().cast::<c_char>()
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_bytes(bytes: *const u8, len: usize) -> u64 {
    if is_invalid_input(bytes, len) {
        return fail_u64();
    }

    if len == 0 {
        return axhash(&[]);
    }

    unsafe { axhash(ffi_bytes_unchecked(bytes, len)) }
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_bytes_seeded(bytes: *const u8, len: usize, seed: u64) -> u64 {
    if is_invalid_input(bytes, len) {
        return fail_u64();
    }

    if len == 0 {
        return axhash_seeded(&[], seed);
    }

    unsafe { axhash_seeded(ffi_bytes_unchecked(bytes, len), seed) }
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_batch_seeded(
    iovecs: *const AxHashIovec,
    count: usize,
    seed: u64,
    out_hashes: *mut u64,
) {
    if iovecs.is_null() || out_hashes.is_null() || count == 0 {
        return;
    }

    let count = core::cmp::min(count, MAX_BATCH);

    unsafe {
        let jobs = core::slice::from_raw_parts(iovecs, count);
        let outs = core::slice::from_raw_parts_mut(out_hashes, count);

        let mut i = 0;
        while i < count {
            let job = jobs.get_unchecked(i);
            let out = outs.get_unchecked_mut(i);

            *out = if is_invalid_input(job.ptr, job.len) {
                0
            } else if job.len == 0 {
                axhash_seeded(&[], seed)
            } else {
                let slice = core::slice::from_raw_parts(job.ptr, job.len);
                axhash_seeded(slice, seed)
            };

            i += 1;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_hasher_new() -> *mut AxHashState {
    as_state_ptr(AxHasher::new())
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_hasher_new_seeded(seed: u64) -> *mut AxHashState {
    as_state_ptr(AxHasher::new_with_seed(seed))
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_hasher_reset(state: *mut AxHashState, seed: u64) -> bool {
    unsafe {
        let Some(hasher) = state_mut(state) else {
            return false;
        };

        *hasher = AxHasher::new_with_seed(seed);
        true
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_hasher_write(
    state: *mut AxHashState,
    bytes: *const u8,
    len: usize,
) -> bool {
    if is_invalid_input(bytes, len) {
        return false;
    }

    unsafe {
        let Some(hasher) = state_mut(state) else {
            return false;
        };

        let slice = core::slice::from_raw_parts(bytes, len);
        hasher.write(slice);
        true
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_hasher_finish(state: *mut AxHashState) -> u64 {
    unsafe {
        let Some(hasher) = state_mut(state) else {
            return 0;
        };

        hasher.finish()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_hasher_free(state: *mut AxHashState) {
    if let Some(state) = NonNull::new(state.cast::<AxHasher>()) {
        unsafe {
            drop(Box::from_raw(state.as_ptr()));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_runtime_backend() -> AxHashRuntimeBackend {
    map_backend(runtime_backend())
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_runtime_has_aes() -> bool {
    runtime_has_aes()
}
