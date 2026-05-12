#![cfg_attr(not(any(feature = "std", test)), no_std)]
mod backend;
mod constants;
pub mod hasher;
mod math;
mod memory;

pub use hasher::AxHasher;
pub use hasher::api::{axhash, axhash_of, axhash_of_seeded, axhash_seeded};
pub use hasher::build::AxBuildHasher;

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeBackend {
    Scalar,
    Aarch64AesNeon,
    X86_64AesAvx2,
}

#[inline(always)]
pub fn runtime_backend() -> RuntimeBackend {
    match backend::selected_backend() {
        backend::Backend::Scalar => RuntimeBackend::Scalar,
        #[cfg(target_arch = "aarch64")]
        backend::Backend::Aarch64AesNeon => RuntimeBackend::Aarch64AesNeon,
        #[cfg(target_arch = "x86_64")]
        backend::Backend::X86_64AesAvx2 => RuntimeBackend::X86_64AesAvx2,
    }
}

#[inline(always)]
pub fn runtime_has_aes() -> bool {
    runtime_backend() != RuntimeBackend::Scalar
}

#[cfg(test)]
mod tests;
