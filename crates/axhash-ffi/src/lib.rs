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

#[inline(always)]
fn as_state_ptr(hasher: AxHasher) -> *mut AxHashState {
    Box::into_raw(Box::new(hasher)).cast::<AxHashState>()
}

#[inline(always)]
unsafe fn with_state_mut<R>(
    state: *mut AxHashState,
    f: impl FnOnce(&mut AxHasher) -> R,
) -> Option<R> {
    let state = NonNull::new(state.cast::<AxHasher>())?;
    Some(f(unsafe { &mut *state.as_ptr() }))
}

#[inline(always)]
fn ffi_bytes(bytes: *const u8, len: usize) -> Option<&'static [u8]> {
    if len == 0 {
        Some(&[])
    } else if bytes.is_null() {
        None
    } else {
        Some(unsafe { core::slice::from_raw_parts(bytes, len) })
    }
}

#[inline(always)]
fn map_backend(backend: RuntimeBackend) -> AxHashRuntimeBackend {
    match backend {
        RuntimeBackend::Scalar => AxHashRuntimeBackend::Scalar,
        RuntimeBackend::Aarch64AesNeon => AxHashRuntimeBackend::Aarch64AesNeon,
        RuntimeBackend::X86_64AesAvx2 => AxHashRuntimeBackend::X86_64AesAvx2,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_ffi_version() -> *const c_char {
    static VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    VERSION.as_ptr().cast::<c_char>()
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_bytes(bytes: *const u8, len: usize) -> u64 {
    ffi_bytes(bytes, len).map(axhash).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_bytes_seeded(bytes: *const u8, len: usize, seed: u64) -> u64 {
    ffi_bytes(bytes, len)
        .map(|slice| axhash_seeded(slice, seed))
        .unwrap_or(0)
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

    unsafe {
        let jobs = core::slice::from_raw_parts(iovecs, count);
        let outs = core::slice::from_raw_parts_mut(out_hashes, count);

        let mut i = 0;

        while i + 3 < count {
            let j0 = jobs.get_unchecked(i);
            let j1 = jobs.get_unchecked(i + 1);
            let j2 = jobs.get_unchecked(i + 2);
            let j3 = jobs.get_unchecked(i + 3);

            *outs.get_unchecked_mut(i) =
                axhash_seeded(core::slice::from_raw_parts(j0.ptr, j0.len), seed);
            *outs.get_unchecked_mut(i + 1) =
                axhash_seeded(core::slice::from_raw_parts(j1.ptr, j1.len), seed);
            *outs.get_unchecked_mut(i + 2) =
                axhash_seeded(core::slice::from_raw_parts(j2.ptr, j2.len), seed);
            *outs.get_unchecked_mut(i + 3) =
                axhash_seeded(core::slice::from_raw_parts(j3.ptr, j3.len), seed);

            i += 4;
        }

        while i < count {
            let job = jobs.get_unchecked(i);
            *outs.get_unchecked_mut(i) =
                axhash_seeded(core::slice::from_raw_parts(job.ptr, job.len), seed);
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
        with_state_mut(state, |hasher| {
            *hasher = AxHasher::new_with_seed(seed);
        })
        .is_some()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_hasher_write(
    state: *mut AxHashState,
    bytes: *const u8,
    len: usize,
) -> bool {
    unsafe {
        with_state_mut(state, |hasher| {
            if let Some(input) = ffi_bytes(bytes, len) {
                hasher.write(input);
                true
            } else {
                false
            }
        })
        .unwrap_or(false)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_hasher_finish(state: *mut AxHashState) -> u64 {
    unsafe { with_state_mut(state, |hasher| hasher.finish()).unwrap_or(0) }
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
