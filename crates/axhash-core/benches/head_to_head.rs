use ahash::RandomState as AHashRandomState;
use axhash_core::{AxBuildHasher, axhash_seeded};
use core::hash::{BuildHasher, Hash, Hasher};
use foldhash::fast::FixedState as FoldHashFixedState;
use rapidhash::fast::SeedableState as RapidSeedableState;
use rapidhash::v3::{RapidSecrets, rapidhash_v3_seeded};
use std::collections::HashMap;
use std::hint::black_box;
use std::time::{Duration, Instant};
use wyhash_final4::generics::WyHasher;
use wyhash_final4::wyhash64::WyHash64;
use xxhash_rust::xxh3::xxh3_64;

const SAMPLE_COUNT: usize = 5;
const WARMUP_ROUNDS: usize = 1;
const AXHASH_SEED: u64 = 0x1234_5678_9abc_def0;
const RAPID_SECRETS: RapidSecrets = RapidSecrets::seed(AXHASH_SEED);

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct MetadataNode {
    id: u64,
    flags: u32,
    session_id: u32,
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
            let remain = bytes.len() - offset;
            bytes[offset..].copy_from_slice(&tail[..remain]);
        }
    }
}

#[derive(Clone)]
struct SampleStats {
    best: Duration,
    avg_secs: f64,
    checksum: u64,
}

#[derive(Clone)]
struct AlgorithmResult {
    algorithm: &'static str,
    stats: SampleStats,
}

#[derive(Clone)]
struct BenchGroup {
    name: &'static str,
    operations: u64,
    total_bytes: u64,
    results: Vec<AlgorithmResult>,
}

#[derive(Clone)]
struct XXH3BuildHasher;

#[derive(Clone)]
struct XXH3Hasher {
    hash: u64,
}

impl std::hash::BuildHasher for XXH3BuildHasher {
    type Hasher = XXH3Hasher;
    fn build_hasher(&self) -> Self::Hasher {
        XXH3Hasher { hash: 0 }
    }
}

impl std::hash::Hasher for XXH3Hasher {
    fn write(&mut self, bytes: &[u8]) {
        self.hash = xxh3_64(bytes);
    }
    fn finish(&self) -> u64 {
        self.hash
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

fn run_algorithm<F>(label: &'static str, mut routine: F) -> AlgorithmResult
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

fn throughput_gib(total_bytes: u64, seconds: f64) -> f64 {
    total_bytes as f64 / seconds / 1_073_741_824.0
}

fn latency_ns(ops: u64, seconds: f64) -> f64 {
    seconds * 1_000_000_000.0 / ops as f64
}

fn print_group(group: &BenchGroup) {
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

fn hash_bytes_via_builder<B: BuildHasher>(builder: &B, bytes: &[u8]) -> u64 {
    let mut hasher = builder.build_hasher();
    hasher.write(bytes);
    hasher.finish()
}

fn bench_short_bytes() -> BenchGroup {
    let (buffers, total_bytes) = build_variable_bytes(1_500_000, 1, 32, 0x1001);
    let ahash = AHashRandomState::with_seed(AXHASH_SEED as usize);
    let foldhash = FoldHashFixedState::with_seed(AXHASH_SEED);
    let rapidhash_state = RapidSeedableState::custom(RAPID_SECRETS.seed, &RAPID_SECRETS.secrets);
    let wyhash = WyHasher::<WyHash64>::from_seed(AXHASH_SEED);

    BenchGroup {
        name: "Short raw bytes (1.5 juta payload 1..32 byte)",
        operations: buffers.len() as u64,
        total_bytes,
        results: vec![
            run_algorithm("axhash", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(axhash_seeded(bytes, AXHASH_SEED));
                }
                acc
            }),
            run_algorithm("rapidhash", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(rapidhash_v3_seeded(bytes, &RAPID_SECRETS));
                }
                acc
            }),
            run_algorithm("ahash", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(hash_bytes_via_builder(&ahash, bytes));
                }
                acc
            }),
            run_algorithm("foldhash", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(hash_bytes_via_builder(&foldhash, bytes));
                }
                acc
            }),
            run_algorithm("wyhash", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(wyhash.hash(bytes));
                }
                acc
            }),
            run_algorithm("rapidhash-h", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(hash_bytes_via_builder(&rapidhash_state, bytes));
                }
                acc
            }),
            run_algorithm("xxh3", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(xxh3_64(bytes));
                }
                acc
            }),
        ],
    }
}

fn bench_mixed_bytes() -> BenchGroup {
    let (buffers, total_bytes) = build_variable_bytes(500_000, 24, 768, 0x2002);
    let ahash = AHashRandomState::with_seed(AXHASH_SEED as usize);
    let foldhash = FoldHashFixedState::with_seed(AXHASH_SEED);
    let rapidhash_state = RapidSeedableState::custom(RAPID_SECRETS.seed, &RAPID_SECRETS.secrets);
    let wyhash = WyHasher::<WyHash64>::from_seed(AXHASH_SEED);

    BenchGroup {
        name: "Mixed raw bytes (500 ribu payload 24..768 byte)",
        operations: buffers.len() as u64,
        total_bytes,
        results: vec![
            run_algorithm("axhash", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(axhash_seeded(bytes, AXHASH_SEED));
                }
                acc
            }),
            run_algorithm("rapidhash", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(rapidhash_v3_seeded(bytes, &RAPID_SECRETS));
                }
                acc
            }),
            run_algorithm("ahash", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(hash_bytes_via_builder(&ahash, bytes));
                }
                acc
            }),
            run_algorithm("foldhash", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(hash_bytes_via_builder(&foldhash, bytes));
                }
                acc
            }),
            run_algorithm("wyhash", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(wyhash.hash(bytes));
                }
                acc
            }),
            run_algorithm("rapidhash-h", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(hash_bytes_via_builder(&rapidhash_state, bytes));
                }
                acc
            }),
            run_algorithm("xxh3", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(xxh3_64(bytes));
                }
                acc
            }),
        ],
    }
}

fn bench_large_bytes() -> BenchGroup {
    let (buffers, total_bytes) = build_variable_bytes(1000, 1024 * 1024, 1024 * 1024, 0x3003);
    let ahash = AHashRandomState::with_seed(AXHASH_SEED as usize);
    let foldhash = FoldHashFixedState::with_seed(AXHASH_SEED);
    let rapidhash_state = RapidSeedableState::custom(RAPID_SECRETS.seed, &RAPID_SECRETS.secrets);
    let wyhash = WyHasher::<WyHash64>::from_seed(AXHASH_SEED);

    BenchGroup {
        name: "Large raw bytes (1000 payload @ 1MB)",
        operations: buffers.len() as u64,
        total_bytes,
        results: vec![
            run_algorithm("axhash", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(axhash_seeded(bytes, AXHASH_SEED));
                }
                acc
            }),
            run_algorithm("rapidhash", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(rapidhash_v3_seeded(bytes, &RAPID_SECRETS));
                }
                acc
            }),
            run_algorithm("ahash", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(hash_bytes_via_builder(&ahash, bytes));
                }
                acc
            }),
            run_algorithm("foldhash", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(hash_bytes_via_builder(&foldhash, bytes));
                }
                acc
            }),
            run_algorithm("wyhash", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(wyhash.hash(bytes));
                }
                acc
            }),
            run_algorithm("rapidhash-h", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(hash_bytes_via_builder(&rapidhash_state, bytes));
                }
                acc
            }),
            run_algorithm("xxh3", || {
                let mut acc = 0u64;
                for bytes in black_box(&buffers) {
                    acc ^= black_box(xxh3_64(bytes));
                }
                acc
            }),
        ],
    }
}

fn bench_struct_hash_trait() -> BenchGroup {
    let data = build_small_struct_dataset(3_000_000);
    let total_bytes = (core::mem::size_of::<MetadataNode>() * data.len()) as u64;
    let axhash = AxBuildHasher::with_seed(AXHASH_SEED);
    let ahash = AHashRandomState::with_seed(AXHASH_SEED as usize);
    let foldhash = FoldHashFixedState::with_seed(AXHASH_SEED);
    let rapidhash = RapidSeedableState::custom(RAPID_SECRETS.seed, &RAPID_SECRETS.secrets);
    let wyhash = WyHasher::<WyHash64>::from_seed(AXHASH_SEED);

    fn hash_structs<B: BuildHasher>(builder: &B, data: &[MetadataNode]) -> u64 {
        let mut acc = 0u64;
        for item in data {
            let mut hasher = builder.build_hasher();
            black_box(item).hash(&mut hasher);
            acc = black_box(acc ^ black_box(hasher.finish()));
        }
        acc
    }

    BenchGroup {
        name: "Struct Hash trait (3 juta struct 16-byte)",
        operations: data.len() as u64,
        total_bytes,
        results: vec![
            run_algorithm("axhash", || {
                black_box(hash_structs(&axhash, black_box(&data)))
            }),
            run_algorithm("rapidhash", || {
                black_box(hash_structs(&rapidhash, black_box(&data)))
            }),
            run_algorithm("ahash", || {
                black_box(hash_structs(&ahash, black_box(&data)))
            }),
            run_algorithm("foldhash", || {
                black_box(hash_structs(&foldhash, black_box(&data)))
            }),
            run_algorithm("wyhash", || {
                black_box(hash_structs(&wyhash, black_box(&data)))
            }),
            run_algorithm("xxh3", || {
                let mut acc = 0u64;
                for item in black_box(&data) {
                    let bytes = unsafe {
                        core::slice::from_raw_parts(
                            (item as *const MetadataNode) as *const u8,
                            core::mem::size_of::<MetadataNode>(),
                        )
                    };
                    acc ^= black_box(xxh3_64(bytes));
                }
                acc
            }),
        ],
    }
}

fn bench_hashmap_insert() -> BenchGroup {
    let data = build_small_struct_dataset(1_000_000);
    let total_bytes = (core::mem::size_of::<MetadataNode>() * data.len()) as u64;

    fn insert_then_probe<B: BuildHasher + Clone>(builder: B, data: &[MetadataNode]) -> u64 {
        let mut map = HashMap::with_capacity_and_hasher(data.len(), builder);
        for (index, item) in data.iter().enumerate() {
            map.insert(*item, index as u32);
        }

        let mut acc = map.len() as u64;
        for item in data.iter().step_by(257) {
            acc ^= *map.get(item).unwrap_or(&0) as u64;
        }
        acc
    }

    BenchGroup {
        name: "HashMap insert pressure (1 juta insert)",
        operations: data.len() as u64,
        total_bytes,
        results: vec![
            run_algorithm("axhash", || {
                black_box(insert_then_probe(
                    AxBuildHasher::with_seed(AXHASH_SEED),
                    black_box(&data),
                ))
            }),
            run_algorithm("rapidhash", || {
                black_box(insert_then_probe(
                    RapidSeedableState::custom(RAPID_SECRETS.seed, &RAPID_SECRETS.secrets),
                    black_box(&data),
                ))
            }),
            run_algorithm("ahash", || {
                black_box(insert_then_probe(
                    AHashRandomState::with_seed(AXHASH_SEED as usize),
                    black_box(&data),
                ))
            }),
            run_algorithm("foldhash", || {
                black_box(insert_then_probe(
                    FoldHashFixedState::with_seed(AXHASH_SEED),
                    black_box(&data),
                ))
            }),
            run_algorithm("wyhash", || {
                black_box(insert_then_probe(
                    WyHasher::<WyHash64>::from_seed(AXHASH_SEED),
                    black_box(&data),
                ))
            }),
            run_algorithm("xxh3", || {
                black_box(insert_then_probe(XXH3BuildHasher, black_box(&data)))
            }),
        ],
    }
}

fn bench_hashmap_lookup() -> BenchGroup {
    let hit_data = build_small_struct_dataset(700_000);
    let miss_data = build_small_struct_dataset(700_000)
        .into_iter()
        .map(|mut item| {
            item.id ^= 0xa5a5_a5a5_a5a5_a5a5;
            item
        })
        .collect::<Vec<_>>();
    let total_bytes =
        ((hit_data.len() + miss_data.len()) * core::mem::size_of::<MetadataNode>()) as u64;

    fn lookup_mix<B: BuildHasher + Clone>(
        builder: B,
        hit: &[MetadataNode],
        miss: &[MetadataNode],
    ) -> u64 {
        let mut map = HashMap::with_capacity_and_hasher(hit.len(), builder);
        for (index, item) in hit.iter().enumerate() {
            map.insert(*item, index as u32);
        }

        let mut acc = 0u64;
        for item in hit {
            acc ^= map.get(item).copied().unwrap_or_default() as u64;
        }
        for item in miss {
            acc ^= map.get(item).copied().unwrap_or_default() as u64;
        }
        acc
    }

    BenchGroup {
        name: "HashMap lookup hit/miss (700 ribu + 700 ribu)",
        operations: (hit_data.len() + miss_data.len()) as u64,
        total_bytes,
        results: vec![
            run_algorithm("axhash", || {
                black_box(lookup_mix(
                    AxBuildHasher::with_seed(AXHASH_SEED),
                    black_box(&hit_data),
                    black_box(&miss_data),
                ))
            }),
            run_algorithm("rapidhash", || {
                black_box(lookup_mix(
                    RapidSeedableState::custom(RAPID_SECRETS.seed, &RAPID_SECRETS.secrets),
                    black_box(&hit_data),
                    black_box(&miss_data),
                ))
            }),
            run_algorithm("ahash", || {
                black_box(lookup_mix(
                    AHashRandomState::with_seed(AXHASH_SEED as usize),
                    black_box(&hit_data),
                    black_box(&miss_data),
                ))
            }),
            run_algorithm("foldhash", || {
                black_box(lookup_mix(
                    FoldHashFixedState::with_seed(AXHASH_SEED),
                    black_box(&hit_data),
                    black_box(&miss_data),
                ))
            }),
            run_algorithm("wyhash", || {
                black_box(lookup_mix(
                    WyHasher::<WyHash64>::from_seed(AXHASH_SEED),
                    black_box(&hit_data),
                    black_box(&miss_data),
                ))
            }),
            run_algorithm("xxh3", || {
                black_box(lookup_mix(
                    XXH3BuildHasher,
                    black_box(&hit_data),
                    black_box(&miss_data),
                ))
            }),
        ],
    }
}

fn print_overall(groups: &[BenchGroup]) {
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

fn main() {
    println!("AXHash head-to-head benchmark");
    println!("CPU              : {}", detect_cpu());
    println!("Sample count     : {}", SAMPLE_COUNT);
    println!("Warmup rounds    : {}", WARMUP_ROUNDS);
    println!("Seed dasar       : 0x{AXHASH_SEED:016x}");
    println!("Metode           : best-of-samples, black_box aktif, dataset tetap per profil");

    let groups = vec![
        bench_short_bytes(),
        bench_mixed_bytes(),
        bench_large_bytes(),
        bench_struct_hash_trait(),
        bench_hashmap_insert(),
        bench_hashmap_lookup(),
    ];

    for group in &groups {
        print_group(group);
    }

    print_overall(&groups);
}
