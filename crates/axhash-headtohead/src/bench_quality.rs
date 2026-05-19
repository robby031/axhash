use crate::hashers::{SEED, hash_entries};

const AVALANCHE_INPUTS: usize = 1_000;
const INPUT_LEN: usize = 32;
const COLLISION_COUNT: usize = 1_000_000;
const DIST_BUCKETS: usize = 256;
const DIST_SAMPLES: usize = 100_000;

fn avalanche_score(hash_fn: fn(&[u8]) -> u64) -> f64 {
    let mut rng = splitmix(SEED);
    let mut total_bit_changes = 0u64;
    let total_possible = (AVALANCHE_INPUTS * INPUT_LEN * 8) as u64;

    let mut buf = [0u8; INPUT_LEN];

    for _ in 0..AVALANCHE_INPUTS {
        for b in buf.iter_mut() {
            *b = (rng.wrapping_add(0x9e3779b97f4a7c15)) as u8;
            rng = rng.wrapping_add(0x9e3779b97f4a7c15);
        }
        let h0 = hash_fn(&buf);

        for byte_idx in 0..INPUT_LEN {
            for bit_idx in 0..8u8 {
                buf[byte_idx] ^= 1 << bit_idx;
                let h1 = hash_fn(&buf);
                buf[byte_idx] ^= 1 << bit_idx;
                total_bit_changes += (h0 ^ h1).count_ones() as u64;
            }
        }
    }

    (total_bit_changes as f64) / (total_possible as f64) / 64.0 * 100.0
}

fn collision_rate(hash_fn: fn(&[u8]) -> u64) -> f64 {
    let mut hashes: Vec<u64> = (0..COLLISION_COUNT as u64)
        .map(|i| {
            let bytes = i.to_le_bytes();
            hash_fn(&bytes)
        })
        .collect();
    hashes.sort_unstable();
    let collisions = hashes.windows(2).filter(|w| w[0] == w[1]).count();
    (collisions as f64) / (COLLISION_COUNT as f64) * 100.0
}

fn distribution_score(hash_fn: fn(&[u8]) -> u64) -> f64 {
    let mut buckets = [0u64; DIST_BUCKETS];
    let mut rng = splitmix(SEED ^ 0x1234);

    for _ in 0..DIST_SAMPLES {
        rng = rng.wrapping_add(0x9e3779b97f4a7c15);
        let bytes = rng.to_le_bytes();
        let h = hash_fn(&bytes);
        buckets[(h & (DIST_BUCKETS as u64 - 1)) as usize] += 1;
    }

    let expected = DIST_SAMPLES as f64 / DIST_BUCKETS as f64;
    let chi2: f64 = buckets
        .iter()
        .map(|&c| {
            let diff = c as f64 - expected;
            diff * diff / expected
        })
        .sum();
    chi2
}

fn bit_bias(hash_fn: fn(&[u8]) -> u64) -> f64 {
    let mut bit_counts = [0u64; 64];
    let n = 500_000u64;
    for i in 0..n {
        let bytes = i.to_le_bytes();
        let h = hash_fn(&bytes);
        for b in 0..64usize {
            bit_counts[b] += (h >> b) & 1;
        }
    }
    let expected = n as f64 * 0.5;
    let max_bias = bit_counts
        .iter()
        .map(|&c| ((c as f64 - expected) / expected).abs())
        .fold(0.0f64, f64::max);
    max_bias * 100.0
}

fn splitmix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

pub fn run() {
    println!("\n=== Hash Quality ===");
    println!("avalanche: % of output bits changed per input bit flip (ideal ≈ 50%)");
    println!("collision: % of 1M sequential u64 inputs that collide (ideal = 0%)");
    println!("chi2:      chi-squared over 256 buckets from 100K samples (ideal ≈ 255)");
    println!("bit_bias:  max per-bit bias across 500K samples (ideal = 0%)");
    println!();
    println!("{:<14}  {:>10}  {:>10}  {:>10}  {:>10}", "Hasher", "avalanche%", "collision%", "chi2", "bit_bias%");
    println!("{}", "─".repeat(60));

    for entry in hash_entries() {
        let av = avalanche_score(entry.hash_bytes);
        let col = collision_rate(entry.hash_bytes);
        let chi = distribution_score(entry.hash_bytes);
        let bias = bit_bias(entry.hash_bytes);
        println!(
            "{:<14}  {:>10.2}  {:>10.6}  {:>10.1}  {:>10.4}",
            entry.name, av, col, chi, bias
        );
    }
}
