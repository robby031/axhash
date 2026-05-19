mod util;

use criterion::{criterion_group, criterion_main};
use util::{SMALL_SIZES, bench_id, configure_criterion, make_data};

const SEED: u64 = 0x123456789ABCDEF0;

fn bench_small_key_latency(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("latency/small-key");

    for &size in SMALL_SIZES {
        let data = make_data(size, SEED ^ size as u64);
        group.bench_with_input(bench_id("axhash", size), &data, |b, d| {
            b.iter(|| axhash::axhash(std::hint::black_box(d)));
        });
        group.bench_with_input(bench_id("axhash_seeded", size), &data, |b, d| {
            b.iter(|| axhash::axhash_seeded(std::hint::black_box(d), SEED));
        });
    }

    group.finish();
}

fn bench_hasher_finish(c: &mut criterion::Criterion) {
    use std::hash::Hasher as _;

    let mut group = c.benchmark_group("latency/hasher-finish");

    group.bench_function("u64-key", |b| {
        b.iter(|| {
            let mut h = axhash::AxHasher::new_with_seed(SEED);
            h.write_u64(std::hint::black_box(0xCAFE_BABE_1234_5678));
            h.finish()
        });
    });

    group.bench_function("u32-key", |b| {
        b.iter(|| {
            let mut h = axhash::AxHasher::new_with_seed(SEED);
            h.write_u32(std::hint::black_box(0xDEAD_BEEF));
            h.finish()
        });
    });

    group.bench_function("str-key-8", |b| {
        let key = b"axhash!!";
        b.iter(|| {
            let mut h = axhash::AxHasher::new_with_seed(SEED);
            h.write(std::hint::black_box(key));
            h.finish()
        });
    });

    group.bench_function("str-key-16", |b| {
        let key = b"axhash-bench-key";
        b.iter(|| {
            let mut h = axhash::AxHasher::new_with_seed(SEED);
            h.write(std::hint::black_box(key));
            h.finish()
        });
    });

    group.finish();
}

fn bench_finish_idempotent(c: &mut criterion::Criterion) {
    use std::hash::Hasher as _;

    let mut group = c.benchmark_group("latency/finish-idempotent");

    group.bench_function("finish-once", |b| {
        let mut h = axhash::AxHasher::new_with_seed(SEED);
        h.write_u64(std::hint::black_box(0xABCD));
        b.iter(|| std::hint::black_box(h.finish()));
    });

    group.bench_function("finish-twice", |b| {
        let mut h = axhash::AxHasher::new_with_seed(SEED);
        h.write_u64(std::hint::black_box(0xABCD));
        b.iter(|| {
            let a = h.finish();
            let b2 = h.finish();
            std::hint::black_box((a, b2))
        });
    });

    group.finish();
}

criterion_group! {
    name = latency_benches;
    config = configure_criterion();
    targets =
        bench_small_key_latency,
        bench_hasher_finish,
        bench_finish_idempotent,
}

criterion_main!(latency_benches);
