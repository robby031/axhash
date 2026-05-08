#![allow(dead_code)]
use criterion::{BenchmarkId, Criterion, Throughput};
use std::time::Duration;

pub const TINY: usize = 4;
pub const SHORT: usize = 16;
pub const MEDIUM: usize = 64;
pub const LARGE: usize = 256;
pub const BULK: usize = 4096;
pub const HUGE: usize = 65536;

pub const ALL_SIZES: &[usize] = &[TINY, SHORT, MEDIUM, LARGE, BULK, HUGE];
pub const SMALL_SIZES: &[usize] = &[TINY, SHORT, MEDIUM, LARGE];
pub const BULK_SIZES: &[usize] = &[LARGE, BULK, HUGE];

pub struct SplitMix64(pub u64);

impl SplitMix64 {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    pub fn next_byte(&mut self) -> u8 {
        self.next() as u8
    }
}

pub fn make_data(len: usize, seed: u64) -> Vec<u8> {
    let mut rng = SplitMix64(seed);
    (0..len).map(|_| rng.next_byte()).collect()
}

pub fn make_arena(count: usize, chunk_len: usize, seed: u64) -> Vec<u8> {
    make_data(count * chunk_len, seed)
}

pub fn configure_criterion() -> Criterion {
    Criterion::default()
        .measurement_time(Duration::from_secs(5))
        .warm_up_time(Duration::from_secs(2))
        .sample_size(200)
        .with_plots()
}

pub fn size_label(n: usize) -> String {
    if n >= 1024 {
        format!("{}K", n / 1024)
    } else {
        format!("{n}B")
    }
}

pub fn bench_id(name: &str, size: usize) -> BenchmarkId {
    BenchmarkId::new(name, size_label(size))
}

pub fn throughput(size: usize) -> Throughput {
    Throughput::Bytes(size as u64)
}
