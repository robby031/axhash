#![cfg_attr(not(any(feature = "std", test)), no_std)]
pub mod hash;

mod bytes;
mod constants;
mod math;
mod memory;

// Convenience re-exports so users can write `use axhash_core::AxHasher`
// instead of navigating the full module path.
pub use hash::AxHasher;
pub use hash::api::{axhash, axhash_of, axhash_of_seeded, axhash_seeded};
pub use hash::build::AxBuildHasher;

// The hardware acceleration backend selected at runtime.
//
// This enum is `#[non_exhaustive]`: future versions may add new variants
// (e.g. RISC-V V, ARM SVE2, WASM SIMD). Always include a wildcard arm
// when matching outside this crate.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeBackend {
    // Pure-Rust scalar path, available on all targets.
    Scalar,
    // AArch64 AES + NEON acceleration.
    Aarch64AesNeon,
    // x86-64 AES-NI + AVX2 acceleration.
    X86_64AesAvx2,
}

// Returns the hardware acceleration backend active on this CPU.
//
// The result is determined once at the first call and then cached by the
// OS/runtime. On `no_std` targets the result is determined at compile time
// from `target_feature` flags.
//
// # Example
//
// ```rust
// use axhash_core::{runtime_backend, RuntimeBackend};
//
// match runtime_backend() {
//     RuntimeBackend::X86_64AesAvx2  => println!("AES-NI + AVX2"),
//     RuntimeBackend::Aarch64AesNeon => println!("AES + NEON"),
//     RuntimeBackend::Scalar         => println!("portable scalar"),
//     _ => println!("unknown backend"),
// }
// ```
#[inline(always)]
pub fn runtime_backend() -> RuntimeBackend {
    match bytes::selected_backend() {
        bytes::Backend::Scalar => RuntimeBackend::Scalar,
        #[cfg(target_arch = "aarch64")]
        bytes::Backend::Aarch64AesNeon => RuntimeBackend::Aarch64AesNeon,
        #[cfg(target_arch = "x86_64")]
        bytes::Backend::X86_64AesAvx2 => RuntimeBackend::X86_64AesAvx2,
    }
}

// Returns `true` when hardware AES acceleration is available.
//
// Equivalent to `runtime_backend() != RuntimeBackend::Scalar`.
//
// # Example
//
// ```rust
// use axhash_core::runtime_has_aes;
//
// if runtime_has_aes() {
//     println!("AES acceleration active");
// }
// ```
#[inline(always)]
pub fn runtime_has_aes() -> bool {
    runtime_backend() != RuntimeBackend::Scalar
}

#[cfg(test)]
mod tests {
    use crate::RuntimeBackend;
    use crate::hash::AxHasher;
    use crate::hash::api::*;
    use core::hash::{Hash, Hasher};

    #[derive(Hash)]
    struct DemoRecord {
        id: u64,
        shard: u32,
        flags: u32,
    }

    #[test]
    fn hash_is_deterministic_for_bytes() {
        let data = b"axhash regression seed";
        let a = axhash_seeded(data, 0x1234_5678_9abc_def0);
        let b = axhash_seeded(data, 0x1234_5678_9abc_def0);
        assert_eq!(a, b);
    }

    #[test]
    fn hash_changes_when_seed_changes() {
        let data = b"same payload different seed";
        let a = axhash_seeded(data, 1);
        let b = axhash_seeded(data, 2);
        assert_ne!(a, b);
    }

    #[test]
    fn hash_trait_path_is_deterministic() {
        let record = DemoRecord {
            id: 42,
            shard: 7,
            flags: 3,
        };
        let a = axhash_of_seeded(&record, 0xdead_beef);
        let b = axhash_of_seeded(&record, 0xdead_beef);
        assert_eq!(a, b);
    }

    #[test]
    fn primitive_writes_produce_a_stable_finish() {
        let mut hasher = AxHasher::new_with_seed(0x4444);
        hasher.write_u64(0x0102_0304_0506_0708);
        hasher.write_u32(0xaabb_ccdd);
        hasher.write_u16(0xeeff);
        hasher.write_u8(0x11);
        let value = hasher.finish();
        assert_ne!(value, 0);
    }

    #[test]
    fn test_axhash_default_seed() {
        let data = b"default seed test";
        let a = axhash(data);
        let b = axhash_seeded(data, 0);
        assert_eq!(a, b);
    }

    #[test]
    fn test_axhash_of_default_seed() {
        let record = DemoRecord {
            id: 1,
            shard: 2,
            flags: 3,
        };
        let a = axhash_of(&record);
        let b = axhash_of_seeded(&record, 0);
        assert_eq!(a, b);
    }

    #[test]
    fn runtime_backend_smoke_test() {
        match super::runtime_backend() {
            RuntimeBackend::Scalar
            | RuntimeBackend::Aarch64AesNeon
            | RuntimeBackend::X86_64AesAvx2 => {}
        }
    }

    // --- empty input ---

    #[test]
    fn empty_input_is_deterministic() {
        assert_eq!(axhash(b""), axhash(b""));
        assert_eq!(axhash_seeded(b"", 0), axhash_seeded(b"", 0));
        assert_eq!(
            axhash_seeded(b"", 0xdead_beef),
            axhash_seeded(b"", 0xdead_beef)
        );
    }

    #[test]
    fn empty_input_changes_with_seed() {
        assert_ne!(axhash_seeded(b"", 0), axhash_seeded(b"", 1));
    }

    #[test]
    fn write_empty_bytes_is_noop() {
        // Writing zero bytes must not change hasher output.
        let seed = 0x1234_5678;

        let mut h1 = AxHasher::new_with_seed(seed);
        h1.write_u64(0xabcd);

        let mut h2 = AxHasher::new_with_seed(seed);
        h2.write(b"");
        h2.write_u64(0xabcd);
        h2.write(b"");

        assert_eq!(h1.finish(), h2.finish());
    }

    // --- dispatch boundary lengths ---

    #[test]
    fn boundary_lengths_are_deterministic() {
        // Covers every dispatch branch in hash_bytes_core:
        // 0           → early return
        // 1..=16      → hash_bytes_short
        // 17..=32     → hash_bytes_17_32
        // 33..=64     → hash_bytes_33_64
        // 65..=128    → hash_bytes_65_128
        // 129+        → hash_bytes_long
        let data = vec![0xABu8; 300];
        let boundaries = [
            0usize, 1, 7, 8, 15, 16, 17, 24, 32, 33, 48, 64, 65, 96, 128, 129, 192, 256, 300,
        ];
        for &len in &boundaries {
            let slice = &data[..len];
            assert_eq!(
                axhash(slice),
                axhash(slice),
                "non-deterministic at len={len}"
            );
        }
    }

    #[test]
    fn adjacent_lengths_produce_different_hashes() {
        // Changing length by one byte must change the hash.
        let data = vec![0xCCu8; 300];
        let boundaries = [1, 8, 16, 17, 32, 33, 64, 65, 128, 129];
        for &len in &boundaries {
            let h1 = axhash(&data[..len]);
            let h2 = axhash(&data[..len + 1]);
            assert_ne!(h1, h2, "collision at len={len} vs len={}", len + 1);
        }
    }

    #[test]
    fn finish_is_idempotent() {
        // Calling finish() twice must return the same value and must not
        // mutate the hasher (the clone-free implementation must uphold this).
        let mut hasher = AxHasher::new_with_seed(0x9999);
        hasher.write_u32(0x1234);
        let a = hasher.finish();
        let b = hasher.finish();
        assert_eq!(a, b);
    }

    #[test]
    fn default_equals_new() {
        use core::hash::Hasher as _;
        let mut h1 = AxHasher::default();
        let mut h2 = AxHasher::new();
        h1.write(b"hello");
        h2.write(b"hello");
        assert_eq!(h1.finish(), h2.finish());
    }

    // --- single-byte coverage ---

    #[test]
    fn all_single_bytes_are_distinct() {
        // Every distinct single-byte input must produce a distinct hash.
        // A collision here would indicate a catastrophic mixing failure.
        let hashes: std::collections::HashSet<u64> = (0u8..=255).map(|b| axhash(&[b])).collect();
        assert_eq!(hashes.len(), 256, "collision among single-byte inputs");
    }

    #[test]
    fn single_byte_differs_from_empty() {
        let empty = axhash(b"");
        for b in 0u8..=255 {
            assert_ne!(axhash(&[b]), empty, "byte {b} collides with empty input");
        }
    }

    // --- seed independence ---

    #[test]
    fn distinct_seeds_produce_independent_families() {
        // For a fixed input, 256 distinct seeds must yield 256 distinct hashes.
        let input = b"seed independence check";
        let hashes: std::collections::HashSet<u64> =
            (0u64..256).map(|s| axhash_seeded(input, s)).collect();
        assert_eq!(hashes.len(), 256, "seed collision detected");
    }

    // --- crate-root re-export consistency ---

    #[test]
    fn crate_root_reexports_match_hash_api() {
        // Convenience re-exports at the crate root must produce the same
        // result as the fully-qualified path.
        use crate::{axhash as root_axhash, axhash_seeded as root_seeded};
        let data = b"reexport consistency";
        assert_eq!(root_axhash(data), axhash(data));
        assert_eq!(root_seeded(data, 0xABCD), axhash_seeded(data, 0xABCD));
    }
}
