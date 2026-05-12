use crate::tests::DemoRecord;
use crate::hasher::AxHasher;
use crate::hasher::api::*;
use core::hash::Hasher;

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
    let seed = 0x1234_5678;

    let mut h1 = AxHasher::new_with_seed(seed);
    h1.write_u64(0xabcd);

    let mut h2 = AxHasher::new_with_seed(seed);
    h2.write(b"");
    h2.write_u64(0xabcd);
    h2.write(b"");

    assert_eq!(h1.finish(), h2.finish());
}

#[test]
fn boundary_lengths_are_deterministic() {
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
    let mut hasher = AxHasher::new_with_seed(0x9999);
    hasher.write_u32(0x1234);
    let a = hasher.finish();
    let b = hasher.finish();
    assert_eq!(a, b);
}

#[test]
fn finish_is_idempotent_after_writes() {
    let mut hasher = AxHasher::new_with_seed(0x7777);
    hasher.write_u8(0xAB);
    hasher.write_u16(0xCDEF);
    let a = hasher.finish();
    let b = hasher.finish();
    let c = hasher.finish();
    assert_eq!(a, b);
    assert_eq!(b, c);
}
