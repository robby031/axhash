use crate::tests::{assert_collision_rate, count_collisions};
use crate::hasher::AxHasher;
use core::hash::Hasher;

#[test]
fn collision_audit_short_inputs_0_to_32() {
    let seed = 0x1234_5678_9ABC_DEF0u64;
    let mut hashes = Vec::with_capacity(50_000);
    let mut unique_keys = std::collections::HashSet::new();
    for len in 0..=32usize {
        for i in 0..1500usize {
            let mut key = vec![0u8; len];
            let mut state = i as u64;
            for b in &mut key {
                state = state.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(1);
                *b = state as u8;
            }
            if unique_keys.insert(key.clone()) {
                hashes.push(crate::axhash_seeded(&key, seed));
            }
        }
    }
    let collisions = count_collisions(&hashes);
    assert_collision_rate(
        "short inputs 0..32 (unique keys only)",
        hashes.len(),
        collisions,
        0.001,
    );
}

#[test]
fn collision_audit_zero_filled_buffers() {
    let seed = 0xABCD_EF01_2345_6789u64;
    let mut hashes = Vec::with_capacity(5_000);
    for len in 0..5000usize {
        let key = vec![0u8; len];
        hashes.push(crate::axhash_seeded(&key, seed));
    }
    let collisions = count_collisions(&hashes);
    assert_collision_rate(
        "zero-filled buffers",
        hashes.len(),
        collisions,
        0.001,
    );
}

#[test]
fn collision_audit_repeated_byte_patterns() {
    let seed = 0xDEAD_BEEF_CAFE_BABEu64;
    let mut hashes = Vec::with_capacity(12_800);
    for byte in 0u8..=255 {
        for len in [1usize, 2, 4, 8, 16, 24, 32, 48, 64] {
            let key = vec![byte; len];
            hashes.push(crate::axhash_seeded(&key, seed));
        }
    }
    let collisions = count_collisions(&hashes);
    assert_collision_rate(
        "repeated byte patterns",
        hashes.len(),
        collisions,
        0.001,
    );
}

#[test]
fn collision_audit_adjacent_lengths() {
    let seed = 0x1111_2222_3333_4444u64;
    let mut hashes = Vec::with_capacity(20_000);
    for base_len in [0usize, 1, 7, 8, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129] {
        for delta in 0..100usize {
            let len = base_len + delta;
            let key = format!("prefix{:08x}{:08x}", len, delta);
            hashes.push(crate::axhash_seeded(key.as_bytes(), seed));
        }
    }
    let collisions = count_collisions(&hashes);
    assert_collision_rate(
        "adjacent lengths",
        hashes.len(),
        collisions,
        0.001,
    );
}

#[test]
fn collision_audit_incremental_numeric_keys() {
    let seed = 0xCAFE_BABE_DEAD_BEEFu64;
    let mut hashes = Vec::with_capacity(200_000);
    for i in 0..200_000usize {
        let mut hasher = AxHasher::new_with_seed(seed);
        hasher.write_u64(i as u64);
        hashes.push(hasher.finish());
    }
    let collisions = count_collisions(&hashes);
    assert_collision_rate(
        "incremental numeric keys",
        hashes.len(),
        collisions,
        0.0005,
    );
}

#[test]
fn collision_audit_small_strings() {
    let seed = 0x9999_AAAA_BBBB_CCCCu64;
    let mut hashes = Vec::with_capacity(50_000);
    for i in 0..50_000usize {
        let key = format!("key{:08x}", i);
        hashes.push(crate::axhash_seeded(key.as_bytes(), seed));
    }
    let collisions = count_collisions(&hashes);
    assert_collision_rate(
        "small strings",
        hashes.len(),
        collisions,
        0.0005,
    );
}

#[test]
fn collision_audit_all_2byte_keys() {
    let seed = 0x7777_8888_9999_AAAAu64;
    let mut hashes = Vec::with_capacity(65_536);
    for b0 in 0u8..=255 {
        for b1 in 0u8..=255 {
            let key = [b0, b1];
            hashes.push(crate::axhash_seeded(&key, seed));
        }
    }
    let collisions = count_collisions(&hashes);
    assert_eq!(
        collisions, 0,
        "All 2-byte keys must be collision-free for a 64-bit hash, found {} collisions",
        collisions
    );
}

#[test]
fn collision_audit_lower8_clustering() {
    let seed = 0x5555_6666_7777_8888u64;
    let n = 200_000usize;
    let mut lower8_counts = [0usize; 256];
    for i in 0..n {
        let mut hasher = AxHasher::new_with_seed(seed);
        hasher.write_u64(i as u64);
        let h = hasher.finish();
        lower8_counts[(h & 0xFF) as usize] += 1;
    }
    let max_bucket = lower8_counts.iter().copied().max().unwrap();
    let expected = n / 256;
    let max_dev_ratio = (max_bucket as f64 - expected as f64) / expected as f64;
    assert!(
        max_dev_ratio < 0.20,
        "Lower-8 clustering: max bucket {} vs expected {}, deviation {:.1}%",
        max_bucket,
        expected,
        max_dev_ratio * 100.0
    );
}
