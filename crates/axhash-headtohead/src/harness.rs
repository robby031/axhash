use std::hint::black_box;
use std::time::Instant;

const WARMUP: u32 = 500;
const SAMPLES: usize = 300;
const BATCH: u32 = 200;

pub struct BenchResult {
    pub name: &'static str,
    pub min_ns: f64,
    pub median_ns: f64,
    pub mean_ns: f64,
    pub stddev_ns: f64,
    pub bytes_per_op: usize,
}

impl BenchResult {
    pub fn gbs(&self) -> f64 {
        if self.bytes_per_op == 0 {
            return 0.0;
        }
        self.bytes_per_op as f64 / (self.median_ns * 1e-9) / 1e9
    }

    #[allow(dead_code)]
    pub fn ops_per_sec(&self) -> f64 {
        1e9 / self.median_ns
    }
}

pub fn measure(
    name: &'static str,
    bytes_per_op: usize,
    ops_per_call: usize,
    mut f: impl FnMut() -> u64,
) -> BenchResult {
    for _ in 0..WARMUP {
        black_box(f());
    }

    let mut samples = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let t0 = Instant::now();
        for _ in 0..BATCH {
            black_box(f());
        }
        let ns = t0.elapsed().as_nanos() as f64;
        samples.push(ns / (BATCH as f64 * ops_per_call as f64));
    }

    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let min_ns = samples[0];
    let median_ns = samples[SAMPLES / 2];
    let sum: f64 = samples.iter().sum();
    let mean_ns = sum / SAMPLES as f64;
    let variance = samples.iter().map(|&x| (x - mean_ns).powi(2)).sum::<f64>() / SAMPLES as f64;

    BenchResult {
        name,
        min_ns,
        median_ns,
        mean_ns,
        stddev_ns: variance.sqrt(),
        bytes_per_op,
    }
}

pub fn measure_wall(
    name: &'static str,
    total_ops: usize,
    mut f: impl FnMut() -> u64,
) -> BenchResult {
    black_box(f());

    let mut samples = Vec::with_capacity(50);
    for _ in 0..50 {
        let t0 = Instant::now();
        black_box(f());
        let ns = t0.elapsed().as_nanos() as f64;
        samples.push(ns / total_ops as f64);
    }

    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let min_ns = samples[0];
    let median_ns = samples[25];
    let sum: f64 = samples.iter().sum();
    let mean_ns = sum / 50.0;
    let variance = samples.iter().map(|&x| (x - mean_ns).powi(2)).sum::<f64>() / 50.0;

    BenchResult {
        name,
        min_ns,
        median_ns,
        mean_ns,
        stddev_ns: variance.sqrt(),
        bytes_per_op: 0,
    }
}

pub fn print_throughput_table(title: &str, size_labels: &[&str], rows: &[Vec<BenchResult>]) {
    println!("\n{title}");
    println!("{}", "─".repeat(title.len()));

    let col_w = 10usize;
    let name_w = 14usize;
    print!("{:<name_w$}", "Hasher");
    for lbl in size_labels {
        print!("{:>col_w$}", lbl);
    }
    println!();
    println!("{}", "─".repeat(name_w + col_w * size_labels.len()));

    for row in rows {
        if row.is_empty() {
            continue;
        }
        print!("{:<name_w$}", row[0].name);
        for r in row {
            print!("{:>col_w$.2}", r.gbs());
        }
        println!(" GB/s");
    }
}

pub fn print_latency_table(title: &str, size_labels: &[&str], rows: &[Vec<BenchResult>]) {
    println!("\n{title}");
    println!("{}", "─".repeat(title.len()));

    let col_w = 10usize;
    let name_w = 14usize;
    print!("{:<name_w$}", "Hasher");
    for lbl in size_labels {
        print!("{:>col_w$}", lbl);
    }
    println!();
    println!("{}", "─".repeat(name_w + col_w * size_labels.len()));

    for row in rows {
        if row.is_empty() {
            continue;
        }
        print!("{:<name_w$}", row[0].name);
        for r in row {
            print!("{:>col_w$.2}", r.median_ns);
        }
        println!(" ns");
    }
}

pub fn print_single_table(title: &str, results: &[BenchResult], unit: &str) {
    println!("\n{title}");
    println!("{}", "─".repeat(title.len()));
    println!("{:<14}  {:>10}  {:>10}  {:>10}  {:>10}", "Hasher", "min", "median", "mean", "±stddev");
    println!("{}", "─".repeat(60));
    for r in results {
        println!(
            "{:<14}  {:>10.2}  {:>10.2}  {:>10.2}  {:>10.2}  {unit}",
            r.name, r.min_ns, r.median_ns, r.mean_ns, r.stddev_ns
        );
    }
}
