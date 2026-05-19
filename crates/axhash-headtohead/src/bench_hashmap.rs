use std::collections::HashMap;
use std::hash::BuildHasher;
use std::hint::black_box;

use crate::harness::{BenchResult, measure_wall, print_single_table};
use crate::hashers::{AHashBH, AxBH, FxBH, HwyBuildHasher, SipBH, WyBH, Xxh3BH, SEED};

const MAP_SIZE: usize = 10_000;

struct SplitMix(u64);

impl SplitMix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}

fn build_map<S: BuildHasher + Default>(n: usize) -> HashMap<u64, u64, S> {
    let mut rng = SplitMix(SEED);
    let mut map = HashMap::with_hasher(S::default());
    for _ in 0..n {
        map.insert(rng.next(), rng.next());
    }
    map
}

fn get_hit<S: BuildHasher + Default>(name: &'static str) -> BenchResult {
    let map: HashMap<u64, u64, S> = build_map::<S>(MAP_SIZE);
    let keys: Vec<u64> = map.keys().cloned().collect();

    measure_wall(name, MAP_SIZE, || {
        let mut sum = 0u64;
        for k in &keys {
            sum = sum.wrapping_add(*map.get(black_box(k)).unwrap_or(&0));
        }
        sum
    })
}

fn mixed_workload<S: BuildHasher + Default>(name: &'static str) -> BenchResult {
    const OPS: usize = MAP_SIZE * 10;
    measure_wall(name, OPS, || {
        let mut map: HashMap<u64, u64, S> = HashMap::with_hasher(S::default());
        let mut rng = SplitMix(SEED);
        for _ in 0..MAP_SIZE {
            map.insert(rng.next(), rng.next());
        }
        let keys: Vec<u64> = map.keys().cloned().collect();

        let mut result = 0u64;
        let mut rng2 = SplitMix(SEED ^ 0xAAAA);
        for _ in 0..OPS {
            let op = rng2.next() % 10;
            if op < 7 {
                let k = keys[rng2.next() as usize % keys.len()];
                result = result.wrapping_add(*map.get(black_box(&k)).unwrap_or(&0));
            } else if op < 9 {
                let k = rng2.next() | 0x8000_0000_0000_0000;
                result = result.wrapping_add(map.get(black_box(&k)).is_some() as u64);
            } else {
                map.insert(rng2.next() & 0x7FFF_FFFF_FFFF_FFFF, rng2.next());
            }
        }
        result
    })
}

pub fn run() {
    let get_hit_results = vec![
        get_hit::<AxBH>("axhash"),
        get_hit::<Xxh3BH>("xxh3"),
        get_hit::<WyBH>("wyhash"),
        get_hit::<AHashBH>("ahash"),
        get_hit::<FxBH>("fxhash"),
        get_hit::<SipBH>("siphash-1-3"),
        get_hit::<HwyBuildHasher>("highwayhash"),
    ];

    print_single_table(
        &format!("=== HashMap get-hit ({MAP_SIZE} entries, ns/op per lookup) ==="),
        &get_hit_results,
        "ns",
    );

    let mixed_results = vec![
        mixed_workload::<AxBH>("axhash"),
        mixed_workload::<Xxh3BH>("xxh3"),
        mixed_workload::<WyBH>("wyhash"),
        mixed_workload::<AHashBH>("ahash"),
        mixed_workload::<FxBH>("fxhash"),
        mixed_workload::<SipBH>("siphash-1-3"),
        mixed_workload::<HwyBuildHasher>("highwayhash"),
    ];

    println!("\n(mixed = 70% get-hit, 20% miss, 10% insert)");
    print_single_table(
        &format!("=== HashMap mixed workload ({MAP_SIZE} entries, ns/op) ==="),
        &mixed_results,
        "ns",
    );
}
