use crate::modules::common::*;
use ahash::RandomState as AHashRandomState;
use axhash_core::hash::api::axhash_seeded;
use foldhash::fast::FixedState as FoldHashFixedState;
use rapidhash::fast::SeedableState as RapidSeedableState;
use rapidhash::v3::rapidhash_v3_seeded;
use std::hint::black_box;
use wyhash_final4::generics::WyHasher;
use wyhash_final4::wyhash64::WyHash64;
use xxhash_rust::xxh3::xxh3_64;

pub fn bench() -> BenchGroup {
    let dataset = build_variable_bytes_arena(1000, 1024 * 1024, 1024 * 1024, 0x3003);
    let order = shuffled_indices(dataset.offsets.len(), 0xdeadbeef);
    let ahash = AHashRandomState::with_seed(AXHASH_SEED as usize);
    let foldhash = FoldHashFixedState::with_seed(AXHASH_SEED);
    let rapidhash_state = RapidSeedableState::custom(RAPID_SECRETS.seed, &RAPID_SECRETS.secrets);
    let wyhash = WyHasher::<WyHash64>::from_seed(AXHASH_SEED);

    BenchGroup {
        name: "Large raw bytes (1000 payload @ 1MB)",
        operations: dataset.offsets.len() as u64,
        total_bytes: dataset.total_bytes,

        results: vec![
            run_algorithm("axhash", || {
                let mut acc = 0u64;
                for &i in black_box(&order) {
                    let (start, len) = dataset.offsets[i];
                    let bytes = &dataset.data[start..start + len];
                    acc ^= black_box(axhash_seeded(bytes, AXHASH_SEED));
                }
                acc
            }),
            run_algorithm("rapidhash", || {
                let mut acc = 0u64;
                for &i in black_box(&order) {
                    let (start, len) = dataset.offsets[i];
                    let bytes = &dataset.data[start..start + len];
                    acc ^= black_box(rapidhash_v3_seeded(bytes, &RAPID_SECRETS));
                }
                acc
            }),
            run_algorithm("ahash", || {
                let mut acc = 0u64;
                for &i in black_box(&order) {
                    let (start, len) = dataset.offsets[i];
                    let bytes = &dataset.data[start..start + len];
                    acc ^= black_box(hash_bytes_via_builder(&ahash, bytes));
                }
                acc
            }),
            run_algorithm("foldhash", || {
                let mut acc = 0u64;
                for &i in black_box(&order) {
                    let (start, len) = dataset.offsets[i];
                    let bytes = &dataset.data[start..start + len];
                    acc ^= black_box(hash_bytes_via_builder(&foldhash, bytes));
                }
                acc
            }),
            run_algorithm("wyhash", || {
                let mut acc = 0u64;
                for &i in black_box(&order) {
                    let (start, len) = dataset.offsets[i];
                    let bytes = &dataset.data[start..start + len];
                    acc ^= black_box(wyhash.hash(bytes));
                }
                acc
            }),
            run_algorithm("rapidhash-h", || {
                let mut acc = 0u64;
                for &i in black_box(&order) {
                    let (start, len) = dataset.offsets[i];
                    let bytes = &dataset.data[start..start + len];
                    acc ^= black_box(hash_bytes_via_builder(&rapidhash_state, bytes));
                }
                acc
            }),
            run_algorithm("xxh3", || {
                let mut acc = 0u64;
                for &i in black_box(&order) {
                    let (start, len) = dataset.offsets[i];
                    let bytes = &dataset.data[start..start + len];
                    acc ^= black_box(xxh3_64(bytes));
                }
                acc
            }),
        ],
    }
}
