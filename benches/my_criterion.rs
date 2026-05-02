use axhash::axhash_seeded;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const SEED: u64 = 0x1234_5678_9abc_def0;

fn generate_entropy_data(size: usize, count: usize) -> Vec<Vec<u8>> {
    (0..count)
        .map(|i| {
            let mut v = Vec::with_capacity(size);
            for j in 0..size {
                v.push(((i * 1315423911 + j * 2654435761) % 256) as u8);
            }
            v
        })
        .collect()
}

fn bench_axhash_only(c: &mut Criterion) {
    let sizes = [8usize, 64, 256, 1024, 4096];

    for &size in &sizes {
        let mut group = c.benchmark_group(format!("axhash_only_{}b", size));

        group.throughput(Throughput::Bytes(size as u64));

        let dataset = generate_entropy_data(size, 128);

        group.bench_function("axhash_oneshot", |b| {
            let mut i = 0usize;

            b.iter(|| {
                let data = &dataset[i & 127];
                i += 1;

                let h = axhash_seeded(black_box(data), SEED);
                black_box(h)
            })
        });

        group.bench_function("axhash_streaming_real", |b| {
            let mut i = 0usize;

            b.iter(|| {
                let data = &dataset[i & 127];
                i += 1;

                let mid = data.len() / 2;

                let mut hasher_input = Vec::new();
                hasher_input.extend_from_slice(&data[..mid]);
                hasher_input.extend_from_slice(&data[mid..]);

                let h = axhash_seeded(black_box(&hasher_input), SEED);
                black_box(h)
            })
        });

        group.bench_function("axhash_hotloop", |b| {
            b.iter(|| {
                let mut acc = 0u64;

                for i in 0..128 {
                    let data = &dataset[i];
                    acc ^= axhash_seeded(black_box(data), SEED);
                }

                black_box(acc)
            })
        });

        group.finish();
    }
}

criterion_group!(benches, bench_axhash_only);
criterion_main!(benches);
