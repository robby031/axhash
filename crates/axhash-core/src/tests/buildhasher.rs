use crate::AxBuildHasher;
use crate::RuntimeBackend;
use crate::hasher::AxHasher;
use crate::hasher::api::*;

#[test]
fn default_equals_new() {
    use core::hash::Hasher as _;
    let mut h1 = AxHasher::default();
    let mut h2 = AxHasher::new();
    h1.write(b"hello");
    h2.write(b"hello");
    assert_eq!(h1.finish(), h2.finish());
}

#[test]
fn buildhasher_default_equals_new() {
    use core::hash::{BuildHasher, Hasher};
    let bh1 = AxBuildHasher::default();
    let bh2 = AxBuildHasher::new();
    let mut h1 = bh1.build_hasher();
    let mut h2 = bh2.build_hasher();
    h1.write(b"hello");
    h2.write(b"hello");
    assert_eq!(h1.finish(), h2.finish());
}

#[test]
fn random_seed_is_usable() {
    use core::hash::{BuildHasher, Hasher};
    let bh = AxBuildHasher::random();
    let mut h = bh.build_hasher();
    h.write(b"randomized seed test");
    let _ = h.finish();
}

#[test]
fn random_seed_produces_different_hashes() {
    use core::hash::{BuildHasher, Hasher};
    let bh1 = AxBuildHasher::random();
    let bh2 = AxBuildHasher::random();
    assert_ne!(
        bh1.prepared_seed, bh2.prepared_seed,
        "random seeds collided"
    );
    let mut h1 = bh1.build_hasher();
    let mut h2 = bh2.build_hasher();
    h1.write(b"same payload");
    h2.write(b"same payload");
    assert_ne!(
        h1.finish(),
        h2.finish(),
        "randomized hashers produced identical output"
    );
}

#[test]
fn all_single_bytes_are_distinct() {
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

#[test]
fn distinct_seeds_produce_independent_families() {
    let input = b"seed independence check";
    let hashes: std::collections::HashSet<u64> =
        (0u64..256).map(|s| axhash_seeded(input, s)).collect();
    assert_eq!(hashes.len(), 256, "seed collision detected");
}

#[test]
fn crate_root_reexports_match_hash_api() {
    use crate::{axhash as root_axhash, axhash_seeded as root_seeded};
    let data = b"reexport consistency";
    assert_eq!(root_axhash(data), axhash(data));
    assert_eq!(root_seeded(data, 0xABCD), axhash_seeded(data, 0xABCD));
}

#[test]
fn runtime_backend_smoke_test() {
    match crate::runtime_backend() {
        RuntimeBackend::Scalar | RuntimeBackend::Aarch64AesNeon | RuntimeBackend::X86_64AesAvx2 => {
        }
    }
}
