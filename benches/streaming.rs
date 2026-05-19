mod util;

use criterion::{BenchmarkId, Throughput, criterion_group, criterion_main};
use std::hash::Hasher as _;
use util::{configure_criterion, make_data};

const SEED: u64 = 0xFEEDBEEFCAFEDEAD;

const CHUNK_SIZES: &[usize] = &[1, 4, 8, 16, 32, 64, 128, 256, 1024];
const TOTAL_BYTES: usize = 4096;

fn bench_chunk_size_impact(c: &mut criterion::Criterion) {
    let data = make_data(TOTAL_BYTES, SEED);

    let mut group = c.benchmark_group("streaming/chunk-size");
    group.throughput(Throughput::Bytes(TOTAL_BYTES as u64));

    for &chunk in CHUNK_SIZES {
        group.bench_with_input(
            BenchmarkId::new("write", format!("{chunk}B-chunks")),
            &chunk,
            |b, &chunk_size| {
                b.iter(|| {
                    let mut h = axhash::AxHasher::new_with_seed(SEED);
                    for slice in data.chunks(chunk_size) {
                        h.write(std::hint::black_box(slice));
                    }
                    h.finish()
                });
            },
        );
    }

    group.finish();
}

fn bench_write_primitives(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("streaming/primitives");

    group.bench_function("write_u8-x64", |b| {
        b.iter(|| {
            let mut h = axhash::AxHasher::new_with_seed(SEED);
            for i in 0u8..64 {
                h.write_u8(std::hint::black_box(i));
            }
            h.finish()
        });
    });

    group.bench_function("write_u16-x32", |b| {
        b.iter(|| {
            let mut h = axhash::AxHasher::new_with_seed(SEED);
            for i in 0u16..32 {
                h.write_u16(std::hint::black_box(i));
            }
            h.finish()
        });
    });

    group.bench_function("write_u32-x16", |b| {
        b.iter(|| {
            let mut h = axhash::AxHasher::new_with_seed(SEED);
            for i in 0u32..16 {
                h.write_u32(std::hint::black_box(i));
            }
            h.finish()
        });
    });

    group.bench_function("write_u64-x8", |b| {
        b.iter(|| {
            let mut h = axhash::AxHasher::new_with_seed(SEED);
            for i in 0u64..8 {
                h.write_u64(std::hint::black_box(i));
            }
            h.finish()
        });
    });

    group.bench_function("write_u128-x4", |b| {
        b.iter(|| {
            let mut h = axhash::AxHasher::new_with_seed(SEED);
            for i in 0u128..4 {
                h.write_u128(std::hint::black_box(i));
            }
            h.finish()
        });
    });

    group.finish();
}

#[derive(Hash)]
struct MixedRecord {
    id: u64,
    shard: u32,
    flags: u16,
    tag: u8,
    label: &'static str,
}

fn bench_mixed_struct(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("streaming/mixed-struct");
    group.throughput(Throughput::Elements(1));

    let record = MixedRecord {
        id: 0xDEAD_BEEF_CAFE_BABE,
        shard: 42,
        flags: 0xFF,
        tag: 7,
        label: "production",
    };

    group.bench_function("derive-Hash-via-hasher", |b| {
        use std::hash::Hash;
        b.iter(|| {
            let mut h = axhash::AxHasher::new_with_seed(SEED);
            std::hint::black_box(&record).hash(&mut h);
            h.finish()
        });
    });

    group.bench_function("axhash_of", |b| {
        b.iter(|| axhash::axhash_of(std::hint::black_box(&record)));
    });

    group.bench_function("axhash_of_seeded", |b| {
        b.iter(|| axhash::axhash_of_seeded(std::hint::black_box(&record), SEED));
    });

    group.finish();
}

fn bench_large_streaming(c: &mut criterion::Criterion) {
    let sizes = [4096usize, 65536];

    let mut group = c.benchmark_group("streaming/large");

    for &total in &sizes {
        let data = make_data(total, SEED ^ total as u64);
        group.throughput(Throughput::Bytes(total as u64));

        group.bench_with_input(
            BenchmarkId::new("single-write", format!("{total}B")),
            &data,
            |b, d| {
                b.iter(|| {
                    let mut h = axhash::AxHasher::new_with_seed(SEED);
                    h.write(std::hint::black_box(d));
                    h.finish()
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("axhash-one-shot", format!("{total}B")),
            &data,
            |b, d| {
                b.iter(|| axhash::axhash(std::hint::black_box(d)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("64B-chunks", format!("{total}B")),
            &data,
            |b, d| {
                b.iter(|| {
                    let mut h = axhash::AxHasher::new_with_seed(SEED);
                    for chunk in d.chunks(64) {
                        h.write(std::hint::black_box(chunk));
                    }
                    h.finish()
                });
            },
        );
    }

    group.finish();
}

criterion_group! {
    name = streaming_benches;
    config = configure_criterion();
    targets =
        bench_chunk_size_impact,
        bench_write_primitives,
        bench_mixed_struct,
        bench_large_streaming,
}

criterion_main!(streaming_benches);
