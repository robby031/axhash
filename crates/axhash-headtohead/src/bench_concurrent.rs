use std::hint::black_box;
use std::sync::{Arc, Barrier};
use std::time::Instant;

use crate::hashers::hash_entries;

const OPS_PER_THREAD: usize = 20_000;
const DATA_LEN: usize = 256;
const THREAD_COUNTS: &[usize] = &[1, 2, 4, 8];
const SAMPLES: usize = 20;

fn make_data() -> Vec<u8> {
    (0..DATA_LEN).map(|i| (i ^ 0xAB) as u8).collect()
}

fn measure_concurrent(
    hash_fn: fn(&[u8]) -> u64,
    data: Arc<Vec<u8>>,
    threads: usize,
) -> f64 {
    let mut wall_times = Vec::with_capacity(SAMPLES);

    for _ in 0..SAMPLES {
        let barrier = Arc::new(Barrier::new(threads + 1));

        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let data = Arc::clone(&data);
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    let mut acc = 0u64;
                    for _ in 0..OPS_PER_THREAD {
                        acc = acc.wrapping_add(hash_fn(black_box(&data)));
                    }
                    acc
                })
            })
            .collect();

        barrier.wait();
        let t0 = Instant::now();
        let _sums: Vec<u64> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let elapsed_ns = t0.elapsed().as_nanos() as f64;

        let total_ops = (threads * OPS_PER_THREAD) as f64;
        wall_times.push(total_ops / (elapsed_ns / 1e9));
    }

    wall_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    wall_times[SAMPLES / 2]
}

pub fn run() {
    let data = Arc::new(make_data());

    println!("\n=== Concurrent throughput (ops/sec, median of {SAMPLES} samples) ===");
    println!("(hash_fn applied to 256B payload; independent per-thread, no shared state)");
    println!();

    let col_w = 14usize;
    let name_w = 14usize;
    print!("{:<name_w$}", "Hasher");
    for &t in THREAD_COUNTS {
        print!("{:>col_w$}", format!("{t}T"));
    }
    println!();
    println!("{}", "─".repeat(name_w + col_w * THREAD_COUNTS.len()));

    for entry in hash_entries() {
        print!("{:<name_w$}", entry.name);
        for &threads in THREAD_COUNTS {
            let ops_sec = measure_concurrent(entry.hash_bytes, Arc::clone(&data), threads);
            print!("{:>col_w$.2e}", ops_sec);
        }
        println!();
    }
}
