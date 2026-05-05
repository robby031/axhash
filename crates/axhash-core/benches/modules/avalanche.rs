use crate::modules::common::*;

const HASH_BITS: u64 = 64;

pub fn avalanche_score<F>(hash: F, samples: usize) -> f64
where
    F: Fn(&[u8]) -> u64,
{
    let mut rng = SplitMix64::new(0x123456789abcdef);
    let mut total_flips = 0u64;
    let mut total_tests = 0u64;

    for _ in 0..samples {
        let mut data = [0u8; 32];
        rng.fill_bytes(&mut data);

        let base = hash(&data);

        for bit in 0..(data.len() * 8) {
            let mut modified = data;
            modified[bit / 8] ^= 1 << (bit % 8);

            let h = hash(&modified);

            total_flips += (base ^ h).count_ones() as u64;
            total_tests += HASH_BITS;
        }
    }

    total_flips as f64 / total_tests as f64
}

pub fn print_avalanche(name: &str, score: f64) {
    let status = if (0.45..=0.55).contains(&score) {
        "OK"
    } else {
        "WARN"
    };

    println!("{:<15} avalanche {:>7.4} | {}", name, score, status);
}

pub fn compare_avalanche(algos: &[(&'static str, &dyn Fn(&[u8]) -> u64)], samples: usize) {
    println!("\n== Avalanche comparison ==");

    for (name, hash) in algos {
        let score = avalanche_score(|d| hash(d), samples);
        print_avalanche(name, score);
    }
}
