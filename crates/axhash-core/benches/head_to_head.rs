mod modules {
    pub mod avalanche;
    pub mod common;
    pub mod large_bytes;
    pub mod mixed_bytes;
    pub mod short_bytes;
}

use crate::modules::common::*;
use ahash::RandomState as AHashRandomState;
use axhash_core::hash::api::axhash_seeded;
use foldhash::fast::FixedState as FoldHashFixedState;
use rapidhash::fast::SeedableState as RapidSeedableState;
use rapidhash::v3::rapidhash_v3_seeded;
use wyhash_final4::generics::WyHasher;
use wyhash_final4::wyhash64::WyHash64;
use xxhash_rust::xxh3::xxh3_64;

fn main() {
    println!("AXHash head-to-head benchmark");
    println!("CPU              : {}", modules::common::detect_cpu());
    println!("Sample count     : {}", modules::common::SAMPLE_COUNT);
    println!("Warmup rounds    : {}", modules::common::WARMUP_ROUNDS);
    println!("Seed dasar       : 0x{:016x}", modules::common::AXHASH_SEED);
    println!("Metode           : best-of-samples, black_box aktif, dataset tetap per profil");

    let groups = vec![
        modules::short_bytes::bench(),
        modules::mixed_bytes::bench(),
        modules::large_bytes::bench(),
    ];

    for group in &groups {
        modules::common::print_group(group);
    }

    modules::common::print_overall(&groups);

    let ahash = AHashRandomState::with_seed(0);
    let foldhash = FoldHashFixedState::with_seed(0);
    let rapidhash_state = RapidSeedableState::custom(RAPID_SECRETS.seed, &RAPID_SECRETS.secrets);
    let wy = WyHasher::<WyHash64>::from_seed(0);

    // let a = [0u8; 32];
    // let mut b = a;
    // b[0] ^= 1;

    // let h1 = axhash_seeded(&a, 0);
    // let h2 = axhash_seeded(&b, 0);

    // println!("diff bits: {}", (h1 ^ h2).count_ones());

    modules::avalanche::compare_avalanche(
        &[
            ("axhash", &|d| axhash_seeded(d, 0)),
            ("xxh3", &|d| xxh3_64(d)),
            ("wyhash", &|d| wy.hash(d)),
            ("ahash", &|d| hash_bytes_via_builder(&ahash, d)),
            ("foldhash", &|d| hash_bytes_via_builder(&foldhash, d)),
            ("rapidhash", &|d| rapidhash_v3_seeded(d, &RAPID_SECRETS)),
            ("rapidhash-h", &|d| {
                hash_bytes_via_builder(&rapidhash_state, d)
            }),
        ],
        1000,
    );
}
