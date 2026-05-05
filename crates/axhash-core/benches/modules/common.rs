use core::hash::{BuildHasher, Hasher};
use rapidhash::v3::RapidSecrets;
use std::collections::HashMap;
use std::hint::black_box;
use std::time::{Duration, Instant};

pub const SAMPLE_COUNT: usize = 5;
pub const WARMUP_ROUNDS: usize = 1;
pub const AXHASH_SEED: u64 = 0x1234_5678_9abc_def0;
pub const RAPID_SECRETS: RapidSecrets = RapidSecrets::seed(AXHASH_SEED);

#[derive(Clone)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    pub fn fill_bytes(&mut self, bytes: &mut [u8]) {
        let mut offset = 0;
        while offset + 8 <= bytes.len() {
            bytes[offset..offset + 8].copy_from_slice(&self.next_u64().to_le_bytes());
            offset += 8;
        }

        if offset < bytes.len() {
            let tail = self.next_u64().to_le_bytes();
            let remain = bytes.len() - offset;
            bytes[offset..].copy_from_slice(&tail[..remain]);
        }
    }
}

#[derive(Clone)]
pub struct SampleStats {
    pub best: Duration,
    pub avg_secs: f64,
    pub checksum: u64,
}

#[derive(Clone)]
pub struct AlgorithmResult {
    pub algorithm: &'static str,
    pub stats: SampleStats,
}

#[derive(Clone)]
pub struct BenchGroup {
    pub name: &'static str,
    pub operations: u64,
    pub total_bytes: u64,
    pub results: Vec<AlgorithmResult>,
}

pub fn detect_cpu() -> String {
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

pub fn run_algorithm<F>(label: &'static str, mut routine: F) -> AlgorithmResult
where
    F: FnMut() -> u64,
{
    for _ in 0..WARMUP_ROUNDS {
        black_box(routine());
    }

    let mut best = Duration::MAX;
    let mut total_secs = 0.0;
    let mut checksum = 0u64;

    for _ in 0..SAMPLE_COUNT {
        let start = Instant::now();
        let value = black_box(routine());
        let elapsed = start.elapsed();
        best = best.min(elapsed);
        total_secs += elapsed.as_secs_f64();
        checksum ^= value.rotate_left((elapsed.subsec_nanos() & 63) as u32);
    }

    AlgorithmResult {
        algorithm: label,
        stats: SampleStats {
            best,
            avg_secs: total_secs / SAMPLE_COUNT as f64,
            checksum,
        },
    }
}

pub fn shuffled_indices(len: usize, seed: u64) -> Vec<usize> {
    let mut rng = SplitMix64::new(seed);
    let mut idx: Vec<_> = (0..len).collect();

    for i in (1..len).rev() {
        let j = (rng.next_u64() as usize) % (i + 1);
        idx.swap(i, j);
    }

    idx
}

pub fn throughput_gib(total_bytes: u64, seconds: f64) -> f64 {
    total_bytes as f64 / seconds / 1_073_741_824.0
}

pub fn latency_ns(ops: u64, seconds: f64) -> f64 {
    seconds * 1_000_000_000.0 / ops as f64
}

pub fn print_group(group: &BenchGroup) {
    let mut ordered = group.results.clone();
    ordered.sort_by(|a, b| a.stats.best.cmp(&b.stats.best));
    let winner_secs = ordered[0].stats.best.as_secs_f64();
    let axhash_secs = ordered
        .iter()
        .find(|result| result.algorithm == "axhash")
        .map(|result| result.stats.best.as_secs_f64())
        .unwrap_or(winner_secs);

    println!("\n== {} ==", group.name);
    println!("Operasi        : {}", group.operations);
    println!(
        "Payload total  : {:.2} MiB",
        group.total_bytes as f64 / 1_048_576.0
    );

    for result in &ordered {
        let best_secs = result.stats.best.as_secs_f64();
        let avg_secs = result.stats.avg_secs;
        let winner_ratio = best_secs / winner_secs;
        let ax_ratio = best_secs / axhash_secs;
        println!(
            "{:<14} best {:>8.2} ns/op | avg {:>8.2} ns/op | peak {:>7.2} GiB/s | vs winner {:>5.2}x | vs axhash {:>5.2}x | checksum {:016x}",
            result.algorithm,
            latency_ns(group.operations, best_secs),
            latency_ns(group.operations, avg_secs),
            throughput_gib(group.total_bytes, best_secs),
            winner_ratio,
            ax_ratio,
            result.stats.checksum
        );
    }

    if let Some(axhash) = group.results.iter().find(|r| r.algorithm == "axhash") {
        let ax_ns = latency_ns(group.operations, axhash.stats.best.as_secs_f64());
        let (winner, winner_ns) = group
            .results
            .iter()
            .map(|r| {
                (
                    r.algorithm,
                    latency_ns(group.operations, r.stats.best.as_secs_f64()),
                )
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();

        if winner != "axhash" {
            let diff = ((ax_ns / winner_ns) - 1.0) * 100.0;
            println!(
                ">> axhash lambat {:.1}% dari {} pada grup ini",
                diff, winner
            );
        } else {
            println!(">> axhash yang tercepat pada grup ini");
        }
    }
}
pub struct Dataset {
    pub data: Vec<u8>,
    pub offsets: Vec<(usize, usize)>, // (start, len)
    pub total_bytes: u64,
}

pub fn build_variable_bytes_arena(
    count: usize,
    min_len: usize,
    max_len: usize,
    seed: u64,
    misalign: usize, // 0 = aligned, >0 = offset shift
) -> Dataset {
    let mut rng = SplitMix64::new(seed);

    let span = max_len - min_len + 1;

    // precompute lengths
    let mut lengths = Vec::with_capacity(count);
    let mut total_bytes = 0usize;

    for _ in 0..count {
        let len = min_len + (rng.next_u64() as usize % span);
        lengths.push(len);
        total_bytes += len + misalign;
    }

    // single allocation (arena)
    let mut data = vec![0u8; total_bytes];
    let mut offsets = Vec::with_capacity(count);

    let mut cursor = 0;

    for len in lengths {
        let start = cursor + misalign;

        rng.fill_bytes(&mut data[start..start + len]);

        offsets.push((start, len));
        cursor += len + misalign;
    }

    Dataset {
        data,
        offsets,
        total_bytes: total_bytes as u64,
    }
}

pub fn hash_bytes_via_builder<B: BuildHasher>(builder: &B, bytes: &[u8]) -> u64 {
    let mut hasher = builder.build_hasher();
    hasher.write(bytes);
    hasher.finish()
}

pub fn print_overall(groups: &[BenchGroup]) {
    let mut wins: HashMap<&'static str, usize> = HashMap::new();
    for group in groups {
        if let Some(best) = group.results.iter().min_by_key(|result| result.stats.best) {
            *wins.entry(best.algorithm).or_insert(0) += 1;
        }
    }

    let mut entries = wins.into_iter().collect::<Vec<_>>();
    entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));

    println!("\n== Ringkasan kemenangan profil ==");
    for (name, count) in entries {
        println!("{:<14} {} profil menang", name, count);
    }
}
