mod util;

use axhash::AxBuildHasher;
use criterion::{BenchmarkId, Throughput, criterion_group, criterion_main};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use util::{SplitMix64, configure_criterion, make_data};

const SEED: u64 = 0xc0de_cafe_babe_f00d;
const OPS_PER_THREAD: usize = 10_000;
const THREAD_COUNTS: &[usize] = &[1, 2, 4, 8];

fn bench_parallel_independent(c: &mut criterion::Criterion) {
    let data = Arc::new(make_data(256, SEED));
    let mut group = c.benchmark_group("concurrent/parallel-independent");

    for &threads in THREAD_COUNTS {
        let total_ops = threads * OPS_PER_THREAD;
        group.throughput(Throughput::Elements(total_ops as u64));

        let data = Arc::clone(&data);
        group.bench_with_input(
            BenchmarkId::new("axhash", threads),
            &threads,
            move |b, &t| {
                let data = Arc::clone(&data);
                b.iter(|| {
                    let handles: Vec<_> = (0..t)
                        .map(|tid| {
                            let data = Arc::clone(&data);
                            std::thread::spawn(move || {
                                let mut sum = 0u64;
                                for i in 0..OPS_PER_THREAD {
                                    sum = sum.wrapping_add(axhash::axhash_seeded(
                                        std::hint::black_box(&data),
                                        (tid as u64) ^ (i as u64),
                                    ));
                                }
                                sum
                            })
                        })
                        .collect();
                    handles.into_iter().map(|h| h.join().unwrap()).sum::<u64>()
                });
            },
        );
    }

    group.finish();
}

fn build_shared_map(n: usize) -> Arc<HashMap<u64, u64, AxBuildHasher>> {
    let mut rng = SplitMix64(SEED);
    let mut map = HashMap::with_hasher(AxBuildHasher::with_seed(SEED));
    for _ in 0..n {
        map.insert(rng.next(), rng.next());
    }
    Arc::new(map)
}

fn bench_shared_readonly_map(c: &mut criterion::Criterion) {
    let map = build_shared_map(OPS_PER_THREAD);
    let keys: Arc<Vec<u64>> = Arc::new(map.keys().cloned().collect());
    let map = Arc::new(map);

    let mut group = c.benchmark_group("concurrent/shared-read-only");

    for &threads in THREAD_COUNTS {
        let total_ops = threads * OPS_PER_THREAD;
        group.throughput(Throughput::Elements(total_ops as u64));

        let map = Arc::clone(&map);
        let keys = Arc::clone(&keys);

        group.bench_with_input(
            BenchmarkId::new("get-hit", threads),
            &threads,
            move |b, &t| {
                let map = Arc::clone(&map);
                let keys = Arc::clone(&keys);
                b.iter(|| {
                    let handles: Vec<_> = (0..t)
                        .map(|tid| {
                            let map = Arc::clone(&map);
                            let keys = Arc::clone(&keys);
                            std::thread::spawn(move || {
                                let mut sum = 0u64;
                                let mut rng = SplitMix64(SEED ^ tid as u64);
                                for _ in 0..OPS_PER_THREAD {
                                    let k = keys[rng.next() as usize % keys.len()];
                                    sum = sum.wrapping_add(
                                        *map.get(std::hint::black_box(&k)).unwrap_or(&0),
                                    );
                                }
                                sum
                            })
                        })
                        .collect();
                    handles.into_iter().map(|h| h.join().unwrap()).sum::<u64>()
                });
            },
        );
    }

    group.finish();
}

fn bench_mutex_write_contention(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("concurrent/mutex-write");

    for &threads in THREAD_COUNTS {
        let total_ops = threads * OPS_PER_THREAD;
        group.throughput(Throughput::Elements(total_ops as u64));

        group.bench_with_input(
            BenchmarkId::new("AxBuildHasher", threads),
            &threads,
            |b, &t| {
                b.iter(|| {
                    let map: Arc<Mutex<HashMap<u64, u64, AxBuildHasher>>> = Arc::new(Mutex::new(
                        HashMap::with_hasher(AxBuildHasher::with_seed(SEED)),
                    ));
                    let handles: Vec<_> = (0..t)
                        .map(|tid| {
                            let map = Arc::clone(&map);
                            std::thread::spawn(move || {
                                let mut rng = SplitMix64(SEED ^ tid as u64);
                                for _ in 0..OPS_PER_THREAD {
                                    let k = rng.next();
                                    let v = rng.next();
                                    map.lock().unwrap().insert(k, v);
                                }
                            })
                        })
                        .collect();
                    for h in handles {
                        h.join().unwrap();
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("DefaultHasher", threads),
            &threads,
            |b, &t| {
                let total = OPS_PER_THREAD * t;

                b.iter(|| {
                    let map: Arc<Mutex<HashMap<u64, u64>>> =
                        Arc::new(Mutex::new(HashMap::with_capacity(total)));

                    let handles: Vec<_> = (0..t)
                        .map(|tid| {
                            let map = Arc::clone(&map);

                            std::thread::spawn(move || {
                                let mut rng = SplitMix64(SEED ^ tid as u64);

                                for _ in 0..OPS_PER_THREAD {
                                    let k = rng.next();
                                    let v = rng.next();

                                    map.lock().unwrap().insert(k, v);
                                }
                            })
                        })
                        .collect();

                    for h in handles {
                        h.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_thread_local_hasher(c: &mut criterion::Criterion) {
    let data = Arc::new(make_data(64, SEED));
    let mut group = c.benchmark_group("concurrent/thread-local-hasher");

    for &threads in THREAD_COUNTS {
        let total_ops = threads * OPS_PER_THREAD;
        group.throughput(Throughput::Elements(total_ops as u64));

        let data = Arc::clone(&data);
        group.bench_with_input(
            BenchmarkId::new("AxHasher-per-thread", threads),
            &threads,
            move |b, &t| {
                let data = Arc::clone(&data);
                b.iter(|| {
                    let handles: Vec<_> = (0..t)
                        .map(|tid| {
                            let data = Arc::clone(&data);
                            std::thread::spawn(move || {
                                use std::hash::Hasher as _;
                                let mut sum = 0u64;
                                for i in 0..OPS_PER_THREAD {
                                    let mut h = axhash::AxHasher::new_with_seed(tid as u64);
                                    h.write(std::hint::black_box(&data));
                                    h.write_u64(i as u64);
                                    sum = sum.wrapping_add(h.finish());
                                }
                                sum
                            })
                        })
                        .collect();
                    handles.into_iter().map(|h| h.join().unwrap()).sum::<u64>()
                });
            },
        );
    }

    group.finish();
}

criterion_group! {
    name = concurrent_benches;
    config = configure_criterion();
    targets =
        bench_parallel_independent,
        bench_shared_readonly_map,
        bench_mutex_write_contention,
        bench_thread_local_hasher,
}

criterion_main!(concurrent_benches);
