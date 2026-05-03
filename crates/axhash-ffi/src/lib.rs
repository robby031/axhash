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
    ptr: *const u8,
    len: usize,
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
    // SAFETY: The opaque pointer is only created from Box<AxHasher> in this crate.
    Some(f(unsafe { &mut *state.as_ptr() }))
}

#[inline(always)]
fn ffi_bytes(bytes: *const u8, len: usize) -> Option<&'static [u8]> {
    if len == 0 {
        Some(&[])
    } else if bytes.is_null() {
        None
    } else {
        // SAFETY: Caller provides a readable buffer of len bytes for non-zero len.
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

        for i in 0..count {
            let job = &jobs[i];
            outs[i] = ffi_bytes(job.ptr, job.len)
                .map(|slice| axhash_seeded(slice, seed))
                .unwrap_or(0);
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
            let Some(input) = ffi_bytes(bytes, len) else {
                return false;
            };
            hasher.write(input);
            true
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
        // SAFETY: The opaque pointer originates from Box::into_raw in this crate.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffi_one_shot_matches_core_api() {
        let data = b"ffi parity";
        let rust_hash = axhash_seeded(data, 99);
        let ffi_hash = axhash_bytes_seeded(data.as_ptr(), data.len(), 99);
        assert_eq!(rust_hash, ffi_hash);
    }

    #[test]
    fn ffi_streaming_matches_core_api() {
        let state = axhash_hasher_new_seeded(7);
        assert!(axhash_hasher_write(state, b"hello ".as_ptr(), 6));
        assert!(axhash_hasher_write(state, b"world".as_ptr(), 5));

        let mut rust_hasher = AxHasher::new_with_seed(7);
        rust_hasher.write(b"hello ");
        rust_hasher.write(b"world");

        assert_eq!(axhash_hasher_finish(state), rust_hasher.finish());
        axhash_hasher_free(state);
    }

    #[test]
    fn ffi_batch_matches_scalar() {
        let str1 = b"apple";
        let str2 = b"banana";
        let str3 = b"cherry";

        let jobs = [
            AxHashIovec {
                ptr: str1.as_ptr(),
                len: str1.len(),
            },
            AxHashIovec {
                ptr: str2.as_ptr(),
                len: str2.len(),
            },
            AxHashIovec {
                ptr: str3.as_ptr(),
                len: str3.len(),
            },
        ];

        let mut outs = [0u64; 3];

        axhash_batch_seeded(jobs.as_ptr(), jobs.len(), 42, outs.as_mut_ptr());

        assert_eq!(outs[0], axhash_bytes_seeded(str1.as_ptr(), str1.len(), 42));
        assert_eq!(outs[1], axhash_bytes_seeded(str2.as_ptr(), str2.len(), 42));
        assert_eq!(outs[2], axhash_bytes_seeded(str3.as_ptr(), str3.len(), 42));
    }
}
