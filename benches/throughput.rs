mod util;

use criterion::{criterion_group, criterion_main};
use util::{ALL_SIZES, bench_id, configure_criterion, make_data, throughput};

const SEED: u64 = 0xdead_cafe_1234_5678;

fn bench_axhash(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("axhash/one-shot");

    for &size in ALL_SIZES {
        let data = make_data(size, SEED);
        group.throughput(throughput(size));
        group.bench_with_input(bench_id("axhash", size), &data, |b, d| {
            b.iter(|| axhash::axhash(std::hint::black_box(d)));
        });
    }

    group.finish();
}

fn bench_axhash_seeded(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("axhash/seeded");

    for &size in ALL_SIZES {
        let data = make_data(size, SEED ^ size as u64);
        group.throughput(throughput(size));
        group.bench_with_input(bench_id("axhash_seeded", size), &data, |b, d| {
            b.iter(|| axhash::axhash_seeded(std::hint::black_box(d), SEED));
        });
    }

    group.finish();
}

#[derive(Hash)]
struct Record {
    id: u64,
    tag: u32,
    flags: u16,
}

fn bench_axhash_of(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("axhash/hash-trait");

    let record = Record {
        id: 0xABCD,
        tag: 7,
        flags: 0x0F,
    };
    group.throughput(criterion::Throughput::Elements(1));

    group.bench_function("axhash_of/struct", |b| {
        b.iter(|| axhash::axhash_of(std::hint::black_box(&record)));
    });

    group.bench_function("axhash_of_seeded/struct", |b| {
        b.iter(|| axhash::axhash_of_seeded(std::hint::black_box(&record), SEED));
    });

    group.bench_function("axhash_of/u64", |b| {
        b.iter(|| axhash::axhash_of(std::hint::black_box(&0xCAFE_BABEu64)));
    });

    group.bench_function("axhash_of/string", |b| {
        let s = "axhash throughput benchmark string key";
        b.iter(|| axhash::axhash_of(std::hint::black_box(&s)));
    });

    group.finish();
}

fn bench_wrapper_api(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("axhash/wrapper-api");

    for &size in ALL_SIZES {
        let data = make_data(size, SEED ^ (size as u64 * 7));
        group.throughput(throughput(size));
        group.bench_with_input(bench_id("hash", size), &data, |b, d| {
            b.iter(|| axhash::hash(std::hint::black_box(d)));
        });
        group.bench_with_input(bench_id("hash_with_seed", size), &data, |b, d| {
            b.iter(|| axhash::hash_with_seed(std::hint::black_box(d), SEED));
        });
    }

    group.finish();
}

criterion_group! {
    name = throughput_benches;
    config = configure_criterion();
    targets =
        bench_axhash,
        bench_axhash_seeded,
        bench_axhash_of,
        bench_wrapper_api,
}

criterion_main!(throughput_benches);
