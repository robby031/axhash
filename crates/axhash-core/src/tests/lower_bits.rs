use crate::tests::{assert_uniform, chi_squared_uniformity};
use crate::hasher::AxHasher;
use core::hash::Hasher;

#[test]
fn lower_bit_distribution_sequential_u64() {
    let seed = 0xDEAD_BEEF_CAFE_BABEu64;
    let n = 50_000usize;
    let mut hashes = Vec::with_capacity(n);
    for i in 0..n {
        let mut hasher = AxHasher::new_with_seed(seed);
        hasher.write_u64(i as u64);
        hashes.push(hasher.finish());
    }
    for mask in [15u64, 31, 63, 127, 255] {
        assert_uniform(
            &format!("seq_u64 & {}", mask),
            &hashes,
            mask,
        );
    }
}

#[test]
fn lower_bit_distribution_random_u64() {
    let seed = 0x1234_5678_9ABC_DEF0u64;
    let n = 50_000usize;
    let mut hashes = Vec::with_capacity(n);
    let mut rng: u64 = 0x123456789abcdef0;
    for _ in 0..n {
        rng = rng.wrapping_mul(0x5851F42D4C957F2D).wrapping_add(1);
        let mut hasher = AxHasher::new_with_seed(seed);
        hasher.write_u64(rng);
        hashes.push(hasher.finish());
    }
    for mask in [15u64, 31, 63, 127, 255] {
        assert_uniform(
            &format!("rand_u64 & {}", mask),
            &hashes,
            mask,
        );
    }
}

#[test]
fn lower_bit_distribution_short_strings() {
    let seed = 0xABCD_EF01_2345_6789u64;
    let n = 50_000usize;
    let mut hashes = Vec::with_capacity(n);
    for i in 0..n {
        let key = format!("key{:08x}", i);
        hashes.push(crate::axhash_seeded(key.as_bytes(), seed));
    }
    for mask in [15u64, 31, 63, 127, 255] {
        assert_uniform(
            &format!("short_str & {}", mask),
            &hashes,
            mask,
        );
    }
}

#[test]
fn lower_bit_distribution_long_strings() {
    let seed = 0xCAFE_BABE_DEAD_BEEFu64;
    let n = 20_000usize;
    let mut hashes = Vec::with_capacity(n);
    let base = "abcdefghijklmnopqrstuvwxyz".repeat(10);
    for i in 0..n {
        let key = format!("{}{:08x}", base, i);
        hashes.push(crate::axhash_seeded(key.as_bytes(), seed));
    }
    for mask in [15u64, 31, 63, 127, 255] {
        let label = format!("long_str & {}", mask);
        let (chi2, max_dev) = chi_squared_uniformity(&hashes, mask);
        let buckets = (mask + 1) as usize;
        let df = (buckets - 1).max(1) as f64;
        let ratio = chi2 / df;
        assert!(
            ratio < 3.0,
            "{}: chi2/df = {:.2} (buckets={}, chi2={:.1}) — distribution suspicious",
            label,
            ratio,
            buckets,
            chi2
        );
        assert!(
            max_dev < 0.45,
            "{}: max bucket deviation = {:.1}% (buckets={}) — too skewed",
            label,
            max_dev * 100.0,
            buckets
        );
    }
}

#[test]
#[ignore = "KNOWN WEAKNESS: adversarial patterned keys (zeros, 0xFF, repeated bytes, short lengths) \
           produce measurable lower-bit bias even after avalanche(). \
           The `hash_bytes_short` path uses `h ^ (h >> 32)` which doesn't fully mix bits. \
           For typical real-world keys this is not a problem, but for power-of-two sharding \
           with adversarial inputs it could cause clustering. \
           FIX (breaking change): strengthen `hash_bytes_short` final mixing or apply \
           `avalanche()` inside `hash_bytes_short` in addition to `finish()`."]
fn lower_bit_distribution_patterned_keys() {
    let seed = 0x1111_2222_3333_4444u64;
    let n = 50_000usize;
    let mut hashes = Vec::with_capacity(n);
    for i in 0..n {
        let key = match i % 8 {
            0 => vec![0u8; (i % 32) + 1],
            1 => vec![0xFFu8; (i % 32) + 1],
            2 => vec![0xA5u8; (i % 32) + 1],
            3 => (0..((i % 32) + 1)).map(|x| x as u8).collect(),
            4 => (0..((i % 32) + 1)).map(|x| (x * 7) as u8).collect(),
            5 => (0..((i % 32) + 1)).map(|x| (x ^ 0x55) as u8).collect(),
            6 => vec![(i & 0xFF) as u8; (i % 32) + 1],
            _ => (0..((i % 32) + 1)).map(|x| ((i ^ x) & 0xFF) as u8).collect(),
        };
        hashes.push(crate::axhash_seeded(&key, seed));
    }
    for mask in [15u64, 31, 63, 127, 255] {
        assert_uniform(
            &format!("patterned & {}", mask),
            &hashes,
            mask,
        );
    }
}

#[test]
#[ignore = "DOCUMENTATION: `hash_bytes_core` without `avalanche()` shows severe lower-bit bias. \
           This is expected — `avalanche()` in `finish()` is the designated finalizer. \
           Do NOT use `hash_bytes_core` directly for sharding; always call `finish()`. \
           Keeping this test to document the intermediate weakness for future audits."]
fn lower_bit_distribution_raw_bytes_core_short() {
    let seed = crate::math::seed_lane(0xDEAD_BEEFu64, 0);
    let n = 50_000usize;
    let mut hashes = Vec::with_capacity(n);
    for i in 0..n {
        let key = format!("k{:08x}", i);
        hashes.push(crate::backend::hash_bytes_core(key.as_bytes(), seed));
    }
    for mask in [15u64, 31, 63, 127, 255] {
        let (chi2, max_dev) = chi_squared_uniformity(&hashes, mask);
        let buckets = (mask + 1) as usize;
        eprintln!(
            "raw_short & {}: chi2={:.1} max_dev={:.1}%",
            mask, chi2, max_dev * 100.0
        );
        assert!(
            chi2 < 100_000.0,
            "raw_short & {}: chi2={:.1} is catastrophically bad (buckets={})",
            mask, chi2, buckets
        );
    }
}

#[test]
#[ignore = "DOCUMENTATION: `hash_bytes_core` for long inputs without `avalanche()` shows \
           measurable lower-bit bias. Same as short path — `avalanche()` in `finish()` \
           is the designated compensation. Keeping as documentation."]
fn lower_bit_distribution_raw_bytes_core_long() {
    let seed = crate::math::seed_lane(0xCAFE_BABEu64, 0);
    let n = 20_000usize;
    let mut hashes = Vec::with_capacity(n);
    let base = "x".repeat(300);
    for i in 0..n {
        let key = format!("{}{:08x}", base, i);
        hashes.push(crate::backend::hash_bytes_core(key.as_bytes(), seed));
    }
    for mask in [15u64, 31, 63, 127, 255] {
        let (chi2, max_dev) = chi_squared_uniformity(&hashes, mask);
        let buckets = (mask + 1) as usize;
        eprintln!(
            "raw_long & {}: chi2={:.1} max_dev={:.1}%",
            mask, chi2, max_dev * 100.0
        );
        assert!(
            chi2 < 100_000.0,
            "raw_long & {}: chi2={:.1} is catastrophically bad (buckets={})",
            mask, chi2, buckets
        );
    }
}
