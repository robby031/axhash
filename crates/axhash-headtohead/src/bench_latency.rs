use std::hint::black_box;

use crate::harness::{BenchResult, measure, print_latency_table};
use crate::hashers::hash_entries;

const SIZES: &[usize] = &[1, 4, 8, 16, 32, 64];
const SIZE_LABELS: &[&str] = &["1B", "4B", "8B", "16B", "32B", "64B"];

fn make_data(len: usize) -> Vec<u8> {
    (0..len).map(|i| i as u8).collect()
}

pub fn run() {
    let datasets: Vec<Vec<u8>> = SIZES.iter().map(|&s| make_data(s)).collect();

    let mut rows: Vec<Vec<BenchResult>> = Vec::new();

    for entry in hash_entries() {
        let mut row = Vec::new();
        for (i, data) in datasets.iter().enumerate() {
            let size = SIZES[i];
            let result = measure(
                entry.name,
                size,
                1,
                || (entry.hash_bytes)(black_box(data)),
            );
            row.push(result);
        }
        rows.push(row);
    }

    print_latency_table("=== Latency small keys (ns/op) ===", SIZE_LABELS, &rows);
}
