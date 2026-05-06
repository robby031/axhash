// All extern "C" functions in this file intentionally accept raw pointers —
// that is required by the C ABI. Pointer validity is verified via null checks
// before any dereference. Marking these functions `unsafe` would be incorrect
// for a public C API boundary and would make them uncallable from safe Rust.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use axhash_core::hash::AxHasher;
use axhash_core::hash::api::{axhash, axhash_seeded};
use axhash_core::{RuntimeBackend, runtime_backend, runtime_has_aes};
use core::ffi::c_char;
use core::hash::Hasher;
use core::ptr::NonNull;

extern crate alloc;
use alloc::boxed::Box;

pub enum AxHashState {}

#[repr(C)]
pub struct AxHashIovec {
    pub ptr: *const u8,
    pub len: usize,
}

// C-stable mirror of [`axhash_core::RuntimeBackend`].
//
// Discriminant values are fixed and will never change, so existing C switch
// statements remain valid. New variants may be added in future releases;
// C code should always include a `default:` case.
#[non_exhaustive]
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
    Box::into_raw(Box::new(hasher)).cast()
}

#[inline(always)]
unsafe fn state_mut<'a>(state: *mut AxHashState) -> Option<&'a mut AxHasher> {
    NonNull::new(state.cast()).map(|p| unsafe { &mut *p.as_ptr() })
}

#[inline(always)]
unsafe fn slice_from_raw<'a>(ptr: *const u8, len: usize) -> &'a [u8] {
    unsafe { core::slice::from_raw_parts(ptr, len) }
}

#[inline(always)]
fn is_invalid_input(ptr: *const u8, len: usize) -> bool {
    len != 0 && ptr.is_null()
}

impl From<RuntimeBackend> for AxHashRuntimeBackend {
    #[inline(always)]
    fn from(b: RuntimeBackend) -> Self {
        match b {
            RuntimeBackend::Scalar => Self::Scalar,
            RuntimeBackend::Aarch64AesNeon => Self::Aarch64AesNeon,
            RuntimeBackend::X86_64AesAvx2 => Self::X86_64AesAvx2,
            // Forward-compatibility: an unknown backend is reported as scalar.
            _ => Self::Scalar,
        }
    }
}

#[cold]
fn fail_u64() -> u64 {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_ffi_version() -> *const c_char {
    static VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    VERSION.as_ptr().cast()
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_bytes(bytes: *const u8, len: usize) -> u64 {
    if is_invalid_input(bytes, len) {
        return fail_u64();
    }

    if len == 0 {
        return axhash(&[]);
    }

    unsafe { axhash(slice_from_raw(bytes, len)) }
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_bytes_seeded(bytes: *const u8, len: usize, seed: u64) -> u64 {
    if is_invalid_input(bytes, len) {
        return fail_u64();
    }

    if len == 0 {
        return axhash_seeded(&[], seed);
    }

    unsafe { axhash_seeded(slice_from_raw(bytes, len), seed) }
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

    let count = count.min(MAX_BATCH);

    // SAFETY: caller guarantees iovecs and out_hashes point to at least `count`
    // valid elements each. count is clamped to MAX_BATCH above.
    let jobs = unsafe { core::slice::from_raw_parts(iovecs, count) };
    let outs = unsafe { core::slice::from_raw_parts_mut(out_hashes, count) };

    for (job, out) in jobs.iter().zip(outs.iter_mut()) {
        *out = if is_invalid_input(job.ptr, job.len) {
            0
        } else if job.len == 0 {
            axhash_seeded(&[], seed)
        } else {
            // SAFETY: is_invalid_input returned false and len > 0, so ptr is
            // non-null and the slice [ptr, ptr+len) is caller-guaranteed valid.
            unsafe { axhash_seeded(slice_from_raw(job.ptr, job.len), seed) }
        };
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
    let hasher = unsafe { state_mut(state) };
    let Some(hasher) = hasher else {
        return false;
    };

    *hasher = AxHasher::new_with_seed(seed);
    true
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

    if len == 0 {
        return true;
    }

    let hasher = unsafe { state_mut(state) };

    let Some(hasher) = hasher else {
        return false;
    };

    unsafe {
        hasher.write(slice_from_raw(bytes, len));
    }

    true
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_hasher_finish(state: *mut AxHashState) -> u64 {
    let hasher = unsafe { state_mut(state) };
    let Some(hasher) = hasher else {
        return 0;
    };

    hasher.finish()
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_hasher_free(state: *mut AxHashState) {
    if state.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(state.cast::<AxHasher>()));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_runtime_backend() -> AxHashRuntimeBackend {
    runtime_backend().into()
}

#[unsafe(no_mangle)]
pub extern "C" fn axhash_runtime_has_aes() -> bool {
    runtime_has_aes()
}
