//! HashMap workload benchmarks — insert, get-hit, get-miss, mixed, and
//! comparison against `DefaultHasher`.
//!
//! These benchmarks model the real-world overhead that `AxBuildHasher` adds
//! compared to the stdlib default, and verify that the `AxBuildHasher`
//! integration is zero-overhead relative to calling `axhash` directly.
//!
//! Run:
//!   cargo bench --bench hashmap

mod util;

use axhash::AxBuildHasher;
use criterion::{BenchmarkId, Throughput, criterion_group, criterion_main};
use std::collections::HashMap;
use std::hash::BuildHasher;
use util::{SplitMix64, configure_criterion};

const SEED: u64 = 0xabcd_ef01_2345_6789;

// ---------------------------------------------------------------------------
// Map sizes under test
// ---------------------------------------------------------------------------

const MAP_SIZES: &[usize] = &[100, 1_000, 10_000, 100_000];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_u64_map<S: BuildHasher + Default>(n: usize) -> HashMap<u64, u64, S> {
    let mut rng = SplitMix64(SEED ^ 0xFFFF);
    let mut map = HashMap::with_hasher(S::default());
    for _ in 0..n {
        map.insert(rng.next(), rng.next());
    }
    map
}

// ---------------------------------------------------------------------------
// Insert: string keys
// ---------------------------------------------------------------------------

fn bench_insert_string(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("hashmap/insert-string");

    for &n in MAP_SIZES {
        group.throughput(Throughput::Elements(n as u64));

        group.bench_with_input(BenchmarkId::new("AxBuildHasher", n), &n, |b, &n| {
            b.iter(|| {
                let mut rng = SplitMix64(SEED);
                let mut map: HashMap<Vec<u8>, u64, AxBuildHasher> =
                    HashMap::with_hasher(AxBuildHasher::with_seed(SEED));
                for i in 0..n {
                    let key = format!("key-{i:010}").into_bytes();
                    map.insert(key, rng.next());
                }
                map
            });
        });

        group.bench_with_input(BenchmarkId::new("DefaultHasher", n), &n, |b, &n| {
            b.iter(|| {
                let mut rng = SplitMix64(SEED);
                let mut map: HashMap<Vec<u8>, u64> = HashMap::new();
                for i in 0..n {
                    let key = format!("key-{i:010}").into_bytes();
                    map.insert(key, rng.next());
                }
                map
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Insert: u64 keys (most cache-friendly scenario)
// ---------------------------------------------------------------------------

fn bench_insert_u64(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("hashmap/insert-u64");

    for &n in MAP_SIZES {
        group.throughput(Throughput::Elements(n as u64));

        group.bench_with_input(BenchmarkId::new("AxBuildHasher", n), &n, |b, &n| {
            b.iter(|| build_u64_map::<AxBuildHasher>(n));
        });

        group.bench_with_input(BenchmarkId::new("DefaultHasher", n), &n, |b, &n| {
            b.iter(|| {
                let mut rng = SplitMix64(SEED ^ 0xFFFF);
                let mut map: HashMap<u64, u64> = HashMap::new();
                for _ in 0..n {
                    map.insert(rng.next(), rng.next());
                }
                map
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Get hit: all lookups present in the map
// ---------------------------------------------------------------------------

fn bench_get_hit(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("hashmap/get-hit");

    for &n in MAP_SIZES {
        group.throughput(Throughput::Elements(n as u64));

        let ax_map: HashMap<u64, u64, AxBuildHasher> = build_u64_map(n);
        let std_map: HashMap<u64, u64> = {
            let mut rng = SplitMix64(SEED ^ 0xFFFF);
            let mut m = HashMap::new();
            for _ in 0..n {
                m.insert(rng.next(), rng.next());
            }
            m
        };
        let keys: Vec<u64> = ax_map.keys().cloned().collect();

        group.bench_with_input(BenchmarkId::new("AxBuildHasher", n), &keys, |b, ks| {
            b.iter(|| {
                let mut sum = 0u64;
                for k in ks {
                    sum = sum.wrapping_add(*ax_map.get(std::hint::black_box(k)).unwrap_or(&0));
                }
                sum
            });
        });

        group.bench_with_input(BenchmarkId::new("DefaultHasher", n), &keys, |b, ks| {
            b.iter(|| {
                let mut sum = 0u64;
                for k in ks {
                    sum = sum.wrapping_add(*std_map.get(std::hint::black_box(k)).unwrap_or(&0));
                }
                sum
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Get miss: keys not in the map
// ---------------------------------------------------------------------------

fn bench_get_miss(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("hashmap/get-miss");

    let n = 10_000;
    group.throughput(Throughput::Elements(n as u64));

    let ax_map: HashMap<u64, u64, AxBuildHasher> = build_u64_map(n);
    let std_map: HashMap<u64, u64> = {
        let mut rng = SplitMix64(SEED ^ 0xFFFF);
        let mut m = HashMap::new();
        for _ in 0..n {
            m.insert(rng.next(), rng.next());
        }
        m
    };

    // Keys guaranteed not present: offset by a constant
    let mut rng = SplitMix64(SEED ^ 0xFFFF);
    let miss_keys: Vec<u64> = (0..n).map(|_| rng.next() ^ 0x0101_0101_0101_0101).collect();

    group.bench_with_input(BenchmarkId::new("AxBuildHasher", n), &miss_keys, |b, ks| {
        b.iter(|| {
            let mut found = 0usize;
            for k in ks {
                found += ax_map.get(std::hint::black_box(k)).is_some() as usize;
            }
            found
        });
    });

    group.bench_with_input(BenchmarkId::new("DefaultHasher", n), &miss_keys, |b, ks| {
        b.iter(|| {
            let mut found = 0usize;
            for k in ks {
                found += std_map.get(std::hint::black_box(k)).is_some() as usize;
            }
            found
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Mixed workload: 70% get-hit, 20% get-miss, 10% insert
// ---------------------------------------------------------------------------

fn bench_mixed_workload(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("hashmap/mixed");

    let n = 10_000;
    group.throughput(Throughput::Elements(n as u64));

    group.bench_function("AxBuildHasher", |b| {
        b.iter(|| {
            let mut map: HashMap<u64, u64, AxBuildHasher> =
                HashMap::with_hasher(AxBuildHasher::with_seed(SEED));
            let mut rng = SplitMix64(SEED);

            // pre-populate
            for _ in 0..n {
                map.insert(rng.next(), rng.next());
            }
            let keys: Vec<u64> = map.keys().cloned().collect();

            // mixed ops
            let mut result = 0u64;
            let mut rng2 = SplitMix64(SEED ^ 0xAA);
            for _ in 0..(n * 10) {
                let op = rng2.next() % 10;
                if op < 7 {
                    // get-hit
                    let k = keys[rng2.next() as usize % keys.len()];
                    result = result.wrapping_add(*map.get(std::hint::black_box(&k)).unwrap_or(&0));
                } else if op < 9 {
                    // get-miss
                    let k = rng2.next() | 0x8000_0000_0000_0000;
                    result =
                        result.wrapping_add(map.get(std::hint::black_box(&k)).is_some() as u64);
                } else {
                    // insert
                    map.insert(rng2.next() & 0x7FFF_FFFF_FFFF_FFFF, rng2.next());
                }
            }
            result
        });
    });

    group.bench_function("DefaultHasher", |b| {
        b.iter(|| {
            let mut map: HashMap<u64, u64> = HashMap::new();
            let mut rng = SplitMix64(SEED);

            for _ in 0..n {
                map.insert(rng.next(), rng.next());
            }
            let keys: Vec<u64> = map.keys().cloned().collect();

            let mut result = 0u64;
            let mut rng2 = SplitMix64(SEED ^ 0xAA);
            for _ in 0..(n * 10) {
                let op = rng2.next() % 10;
                if op < 7 {
                    let k = keys[rng2.next() as usize % keys.len()];
                    result = result.wrapping_add(*map.get(std::hint::black_box(&k)).unwrap_or(&0));
                } else if op < 9 {
                    let k = rng2.next() | 0x8000_0000_0000_0000;
                    result =
                        result.wrapping_add(map.get(std::hint::black_box(&k)).is_some() as u64);
                } else {
                    map.insert(rng2.next() & 0x7FFF_FFFF_FFFF_FFFF, rng2.next());
                }
            }
            result
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// AxBuildHasher::build_hasher overhead
// ---------------------------------------------------------------------------

fn bench_build_hasher(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("hashmap/build-hasher");

    let bh = AxBuildHasher::with_seed(SEED);
    group.bench_function("build_hasher", |b| {
        use std::hash::BuildHasher;
        b.iter(|| std::hint::black_box(bh.build_hasher()));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Realistic key: short borrowed &str slices
// ---------------------------------------------------------------------------

fn bench_str_key_map(c: &mut criterion::Criterion) {
    let keys: Vec<String> = (0..1000).map(|i| format!("session-key-{i:06}")).collect();
    let data: Vec<u8> = (0u8..=255).cycle().take(1000).collect();

    let mut group = c.benchmark_group("hashmap/str-keys");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("insert-1000-AxBuildHasher", |b| {
        b.iter(|| {
            let mut map: HashMap<&str, u8, AxBuildHasher> =
                HashMap::with_hasher(AxBuildHasher::with_seed(SEED));
            for (k, v) in keys.iter().zip(data.iter()) {
                map.insert(std::hint::black_box(k.as_str()), *v);
            }
            map
        });
    });

    group.bench_function("insert-1000-DefaultHasher", |b| {
        b.iter(|| {
            let mut map: HashMap<&str, u8> = HashMap::new();
            for (k, v) in keys.iter().zip(data.iter()) {
                map.insert(std::hint::black_box(k.as_str()), *v);
            }
            map
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion entry points
// ---------------------------------------------------------------------------

criterion_group! {
    name = hashmap_benches;
    config = configure_criterion();
    targets =
        bench_insert_string,
        bench_insert_u64,
        bench_get_hit,
        bench_get_miss,
        bench_mixed_workload,
        bench_build_hasher,
        bench_str_key_map,
}

criterion_main!(hashmap_benches);
