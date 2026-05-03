use axhash_core::{AxBuildHasher, axhash_of_seeded, axhash_seeded};
use core::hash::Hash;
use std::collections::HashMap;
use std::hint::black_box;
use std::time::{Duration, Instant};

const SAMPLE_COUNT: usize = 6;
const WARMUP_ROUNDS: usize = 2;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct MetadataNode {
    id: u64,
    flags: u32,
    session_id: u32,
}

#[derive(Clone)]
struct SampleStats {
    best: Duration,
    worst: Duration,
    avg_secs: f64,
    checksum: u64,
}

#[derive(Clone)]
struct BenchResult {
    name: &'static str,
    operations: u64,
    total_bytes: u64,
    stats: SampleStats,
}

#[derive(Clone)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn fill_bytes(&mut self, bytes: &mut [u8]) {
        let mut offset = 0;
        while offset + 8 <= bytes.len() {
            bytes[offset..offset + 8].copy_from_slice(&self.next_u64().to_le_bytes());
            offset += 8;
        }

        if offset < bytes.len() {
            let tail = self.next_u64().to_le_bytes();
            let remaining = bytes.len() - offset;
            bytes[offset..].copy_from_slice(&tail[..remaining]);
        }
    }
}

fn detect_cpu() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|text| {
            text.lines()
                .find_map(|line| line.strip_prefix("model name\t: ").map(str::to_owned))
        })
        .or_else(|| {
            std::process::Command::new("sysctl")
                .args(["-n", "machdep.cpu.brand_string"])
                .output()
                .ok()
                .filter(|output| output.status.success())
                .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        })
        .unwrap_or_else(|| "CPU tidak terdeteksi".to_owned())
}

fn run_benchmark<F>(
    name: &'static str,
    operations: u64,
    total_bytes: u64,
    mut routine: F,
) -> BenchResult
where
    F: FnMut() -> u64,
{
    for _ in 0..WARMUP_ROUNDS {
        black_box(routine());
    }

    let mut best = Duration::MAX;
    let mut worst = Duration::ZERO;
    let mut total_secs = 0.0;
    let mut checksum = 0u64;

    for _ in 0..SAMPLE_COUNT {
        let start = Instant::now();
        let value = black_box(routine());
        let elapsed = start.elapsed();

        best = best.min(elapsed);
        worst = worst.max(elapsed);
        total_secs += elapsed.as_secs_f64();
        checksum ^= value.rotate_left((elapsed.subsec_nanos() & 63) as u32);
    }

    BenchResult {
        name,
        operations,
        total_bytes,
        stats: SampleStats {
            best,
            worst,
            avg_secs: total_secs / SAMPLE_COUNT as f64,
            checksum,
        },
    }
}

fn print_result(index: usize, total: usize, result: &BenchResult) {
    let best_ns = result.stats.best.as_nanos() as f64;
    let avg_ns = result.stats.avg_secs * 1_000_000_000.0;
    let worst_ns = result.stats.worst.as_nanos() as f64;
    let per_op_best = best_ns / result.operations as f64;
    let per_op_avg = avg_ns / result.operations as f64;
    let per_op_worst = worst_ns / result.operations as f64;
    let throughput_best =
        result.total_bytes as f64 / result.stats.best.as_secs_f64() / 1_073_741_824.0;
    let throughput_avg = result.total_bytes as f64 / result.stats.avg_secs / 1_073_741_824.0;

    println!("\n[{index}/{total}] {}", result.name);
    println!("      Operasi          : {}", result.operations);
    println!(
        "      Payload total    : {:.2} MiB",
        result.total_bytes as f64 / 1_048_576.0
    );
    println!("      Best time        : {:?}", result.stats.best);
    println!("      Worst time       : {:?}", result.stats.worst);
    println!("      Avg latency      : {:.2} ns/op", per_op_avg);
    println!("      Best latency     : {:.2} ns/op", per_op_best);
    println!("      Worst latency    : {:.2} ns/op", per_op_worst);
    println!("      Avg throughput   : {:.2} GiB/s", throughput_avg);
    println!("      Peak throughput  : {:.2} GiB/s", throughput_best);
    println!("      Checksum         : {:016x}", result.stats.checksum);
}

fn build_small_struct_dataset(count: usize) -> Vec<MetadataNode> {
    (0..count)
        .map(|i| MetadataNode {
            id: (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15),
            flags: ((i * 17) ^ (i >> 3)) as u32,
            session_id: ((i * 131) ^ (i >> 5)) as u32,
        })
        .collect()
}

fn build_variable_bytes(
    count: usize,
    min_len: usize,
    max_len: usize,
    seed: u64,
) -> (Vec<Vec<u8>>, u64) {
    let mut rng = SplitMix64::new(seed);
    let mut buffers = Vec::with_capacity(count);
    let mut total_bytes = 0u64;
    let span = max_len - min_len + 1;

    for _ in 0..count {
        let len = min_len + (rng.next_u64() as usize % span);
        let mut bytes = vec![0u8; len];
        rng.fill_bytes(&mut bytes);
        total_bytes += len as u64;
        buffers.push(bytes);
    }

    (buffers, total_bytes)
}

fn bench_small_struct_hash() -> BenchResult {
    let seed = 0x00ab_cdef_u64;
    let data = build_small_struct_dataset(4_000_000);
    let total_bytes = (core::mem::size_of::<MetadataNode>() * data.len()) as u64;

    run_benchmark(
        "Small-key struct hashing (4 juta struct 16-byte, path `Hash` trait)",
        data.len() as u64,
        total_bytes,
        || {
            let mut acc = 0u64;
            for item in black_box(&data) {
                acc ^= black_box(axhash_of_seeded(item, seed));
            }
            acc
        },
    )
}

fn bench_short_messages() -> BenchResult {
    let seed = 0x1357_9bdf_2468_ace0;
    let (buffers, total_bytes) = build_variable_bytes(2_000_000, 1, 32, 0x1111);

    run_benchmark(
        "Short messages (2 juta payload 1..32 byte, path raw bytes)",
        buffers.len() as u64,
        total_bytes,
        || {
            let mut acc = 0u64;
            for bytes in black_box(&buffers) {
                acc ^= black_box(axhash_seeded(bytes, seed));
            }
            acc
        },
    )
}

fn bench_mixed_api_payloads() -> BenchResult {
    let seed = 0xfeed_face_dead_beef;
    let (buffers, total_bytes) = build_variable_bytes(750_000, 24, 768, 0x2222);

    run_benchmark(
        "API payload campuran (750 ribu payload 24..768 byte, distribusi realistis)",
        buffers.len() as u64,
        total_bytes,
        || {
            let mut acc = 0u64;
            for bytes in black_box(&buffers) {
                acc ^= black_box(axhash_seeded(bytes, seed));
            }
            acc
        },
    )
}

fn bench_bulk_stream() -> BenchResult {
    let seed = 0x7777_aaaa_9999_0001;
    let chunk_len = 2 * 1024 * 1024;
    let chunk_count = 96;
    let mut rng = SplitMix64::new(0x3333);
    let mut chunks = Vec::with_capacity(chunk_count);

    for _ in 0..chunk_count {
        let mut bytes = vec![0u8; chunk_len];
        rng.fill_bytes(&mut bytes);
        chunks.push(bytes);
    }

    let total_bytes = (chunk_len * chunk_count) as u64;

    run_benchmark(
        "Bulk streaming (96 blok x 2 MiB, total 192 MiB)",
        chunk_count as u64,
        total_bytes,
        || {
            let mut acc = 0u64;
            for bytes in black_box(&chunks) {
                acc ^= black_box(axhash_seeded(bytes, seed));
            }
            acc
        },
    )
}

fn bench_hashmap_inserts() -> BenchResult {
    let data = build_small_struct_dataset(1_500_000);
    let total_bytes = (core::mem::size_of::<MetadataNode>() * data.len()) as u64;

    run_benchmark(
        "HashMap insert pressure (1.5 juta insert dengan custom BuildHasher)",
        data.len() as u64,
        total_bytes,
        || {
            let mut map =
                HashMap::with_capacity_and_hasher(data.len(), AxBuildHasher::with_seed(0x4444));
            for (index, item) in black_box(data.iter()).enumerate() {
                map.insert(*item, index as u32);
            }

            let mut acc = map.len() as u64;
            for item in data.iter().step_by(257) {
                acc ^= *map.get(item).unwrap_or(&0) as u64;
            }
            acc
        },
    )
}

fn bench_hashmap_hit_miss() -> BenchResult {
    let hit_data = build_small_struct_dataset(900_000);
    let miss_data = build_small_struct_dataset(900_000)
        .into_iter()
        .map(|mut item| {
            item.id ^= 0xa5a5_a5a5_a5a5_a5a5;
            item
        })
        .collect::<Vec<_>>();

    let total_bytes =
        ((hit_data.len() + miss_data.len()) * core::mem::size_of::<MetadataNode>()) as u64;

    run_benchmark(
        "HashMap lookup hit/miss (900 ribu hit + 900 ribu miss)",
        (hit_data.len() + miss_data.len()) as u64,
        total_bytes,
        || {
            let mut map =
                HashMap::with_capacity_and_hasher(hit_data.len(), AxBuildHasher::with_seed(0x5555));
            for (index, item) in hit_data.iter().enumerate() {
                map.insert(*item, index as u32);
            }

            let mut acc = 0u64;
            for item in black_box(&hit_data) {
                acc ^= map.get(item).copied().unwrap_or_default() as u64;
            }
            for item in black_box(&miss_data) {
                acc ^= map.get(item).copied().unwrap_or_default() as u64;
            }
            acc
        },
    )
}

fn main() {
    println!("AXHash real-world stress bench");
    println!("Versi crate      : {}", env!("CARGO_PKG_VERSION"));
    println!("CPU              : {}", detect_cpu());
    println!("Sample count     : {}", SAMPLE_COUNT);
    println!("Warmup rounds    : {}", WARMUP_ROUNDS);
    println!("Proteksi bench   : std::hint::black_box aktif");

    let results = [
        bench_small_struct_hash(),
        bench_short_messages(),
        bench_mixed_api_payloads(),
        bench_bulk_stream(),
        bench_hashmap_inserts(),
        bench_hashmap_hit_miss(),
    ];

    for (index, result) in results.iter().enumerate() {
        print_result(index + 1, results.len(), result);
    }
}
