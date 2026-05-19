use std::hint::black_box;

use crate::harness::{BenchResult, measure, print_latency_table};
use crate::hashers::stream_entries;

const TOTAL_BYTES: usize = 4096;
const CHUNK_SIZES: &[usize] = &[8, 32, 64, 256, 1024, 4096];
const CHUNK_LABELS: &[&str] = &["8B", "32B", "64B", "256B", "1KB", "4KB"];

fn make_data() -> Vec<u8> {
    (0..TOTAL_BYTES).map(|i| (i ^ (i >> 3)) as u8).collect()
}

pub fn run() {
    let data = make_data();
    let mut rows: Vec<Vec<BenchResult>> = Vec::new();

    for entry in stream_entries() {
        let mut row = Vec::new();
        for (i, &chunk) in CHUNK_SIZES.iter().enumerate() {
            let result = measure(
                entry.name,
                TOTAL_BYTES,
                1,
                || (entry.stream)(black_box(&data), chunk),
            );
            let _ = CHUNK_LABELS[i];
            row.push(result);
        }
        rows.push(row);
    }

    println!("\n=== Streaming 4096B via chunk writes (ns/op total) ===");
    println!("(lower = faster; measures one full 4KB hash via incremental writes)");
    print_latency_table("  chunk size →", CHUNK_LABELS, &rows);
}
