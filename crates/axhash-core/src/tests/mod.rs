#![allow(unused_imports)]

use crate::AxBuildHasher;
use crate::hasher::AxHasher;
use crate::hasher::api::*;
use core::hash::{BuildHasher, Hash, Hasher};

#[derive(Hash)]
pub(crate) struct DemoRecord {
    pub id: u64,
    pub shard: u32,
    pub flags: u32,
}

pub(crate) fn chi_squared_uniformity(hashes: &[u64], mask: u64) -> (f64, f64) {
    let buckets = (mask + 1) as usize;
    let mut counts = vec![0usize; buckets];
    for &h in hashes {
        counts[(h & mask) as usize] += 1;
    }
    let expected = hashes.len() as f64 / buckets as f64;
    let mut chi2 = 0.0;
    let mut max_dev = 0.0f64;
    for &c in &counts {
        let diff = c as f64 - expected;
        chi2 += (diff * diff) / expected;
        let dev = diff.abs() / expected;
        if dev > max_dev {
            max_dev = dev;
        }
    }
    (chi2, max_dev)
}

pub(crate) fn assert_uniform(label: &str, hashes: &[u64], mask: u64) {
    let buckets = (mask + 1) as usize;
    let (chi2, max_dev) = chi_squared_uniformity(hashes, mask);
    let df = (buckets - 1).max(1) as f64;
    let ratio = chi2 / df;
    assert!(
        ratio < 2.5,
        "{}: chi2/df = {:.2} (buckets={}, chi2={:.1}) — distribution suspicious",
        label,
        ratio,
        buckets,
        chi2
    );
    assert!(
        max_dev < 0.30,
        "{}: max bucket deviation = {:.1}% (buckets={}) — too skewed",
        label,
        max_dev * 100.0,
        buckets
    );
}

pub(crate) fn assert_collision_rate(label: &str, total: usize, collisions: usize, max_rate: f64) {
    let rate = collisions as f64 / total as f64;
    assert!(
        rate < max_rate,
        "{}: collision rate {:.4}% ({}/{}) exceeds threshold {:.4}%",
        label,
        rate * 100.0,
        collisions,
        total,
        max_rate * 100.0
    );
}

pub(crate) fn count_collisions(hashes: &[u64]) -> usize {
    use std::collections::HashSet;
    let mut seen = HashSet::with_capacity(hashes.len());
    let mut dups = 0usize;
    for &h in hashes {
        if !seen.insert(h) {
            dups += 1;
        }
    }
    dups
}

mod backend_parity;
mod buildhasher;
mod collisions;
mod determinism;
mod lower_bits;
mod predictability;
mod trait_contract;
