use crate::AxBuildHasher;
use crate::hasher::AxHasher;
use core::hash::{BuildHasher, Hasher};

#[test]
fn hasher_init_consistency_for_seed_zero() {
    let h1 = AxHasher::new();
    let h2 = AxHasher::new_with_seed(0);
    let h3 = AxHasher::default();
    assert_eq!(h1.acc, h2.acc);
    assert_eq!(h1.acc, h3.acc);
    assert_eq!(h1.sponge, h2.sponge);
    assert_eq!(h1.sponge, h3.sponge);
    assert_eq!(h1.sponge_bits, h2.sponge_bits);
    assert_eq!(h1.sponge_bits, h3.sponge_bits);
}

#[test]
fn buildhasher_init_consistency() {
    let b1 = AxBuildHasher::new();
    let b2 = AxBuildHasher::default();
    let b3 = AxBuildHasher::with_seed(0);
    assert_eq!(b1.prepared_seed, b2.prepared_seed);
    assert_eq!(b1.prepared_seed, b3.prepared_seed);
}

#[test]
fn buildhasher_produces_matching_hasher() {
    let bh = AxBuildHasher::with_seed(0x1234_5678_9ABC_DEF0u64);
    let hasher = bh.build_hasher();
    let direct = AxHasher::new_with_seed(0x1234_5678_9ABC_DEF0u64);
    assert_eq!(hasher.acc, direct.acc);
    assert_eq!(hasher.sponge, direct.sponge);
    assert_eq!(hasher.sponge_bits, direct.sponge_bits);
}

#[test]
fn hasher_is_pure_no_hidden_state() {
    let seed = 0xDEAD_BEEFu64;
    let data = b"some arbitrary test data";
    let mut h1 = AxHasher::new_with_seed(seed);
    h1.write(data);
    let r1 = h1.finish();

    let mut h2 = AxHasher::new_with_seed(seed);
    h2.write(data);
    let r2 = h2.finish();

    assert_eq!(
        r1, r2,
        "Hasher must be pure: same seed + same input -> same output"
    );
}

#[test]
fn seed_derivation_inconsistency_documentation() {
    let seed = 0xCAFE_BABEu64;
    let data = b"test";

    let mut hasher = AxHasher::new_with_seed(seed);
    hasher.write(data);
    let h1 = hasher.finish();

    let h2 = crate::axhash_seeded(data, seed);

    assert_ne!(
        h1, h2,
        "DOCUMENTATION: axhash_seeded and AxHasher::new_with_seed use different seed \
         derivation. h1={:016x} h2={:016x}",
        h1, h2
    );
}

#[test]
fn random_seed_produces_unique_instances() {
    let b1 = AxBuildHasher::random();
    let b2 = AxBuildHasher::random();
    assert_ne!(
        b1.prepared_seed, b2.prepared_seed,
        "random() must produce different seeds on successive calls"
    );
}

#[test]
fn buildhasher_is_pure() {
    let bh = AxBuildHasher::with_seed(0x1234_5678u64);
    let h1 = bh.build_hasher();
    let h2 = bh.build_hasher();
    assert_eq!(h1.acc, h2.acc);
    assert_eq!(h1.sponge, h2.sponge);
    assert_eq!(h1.sponge_bits, h2.sponge_bits);
}
