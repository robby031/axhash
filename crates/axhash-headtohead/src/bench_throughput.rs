use std::hint::black_box;

use crate::harness::{BenchResult, measure, print_throughput_table};
use crate::hashers::hash_entries;

const SIZES: &[usize] = &[4, 16, 64, 256, 4096, 65536];
const SIZE_LABELS: &[&str] = &["4B", "16B", "64B", "256B", "4KB", "64KB"];

fn make_data(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i.wrapping_mul(0x9e37) ^ (i >> 5)) as u8).collect()
}

pub fn run() {
    let datasets: Vec<Vec<u8>> = SIZES.iter().map(|&s| make_data(s)).collect();

    let mut rows: Vec<Vec<BenchResult>> = Vec::new();

    for entry in hash_entries() {
        let mut row = Vec::new();
        for (i, data) in datasets.iter().enumerate() {
            let size = SIZES[i];
            let label = SIZE_LABELS[i];
            let result = measure(
                entry.name,
                size,
                1,
                || (entry.hash_bytes)(black_box(data)),
            );
            row.push(BenchResult { name: entry.name, ..result });
            let _ = label;
        }
        rows.push(row);
    }

    print_throughput_table("=== Throughput (GB/s) ===", SIZE_LABELS, &rows);
}
