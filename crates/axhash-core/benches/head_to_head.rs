mod modules {
    pub mod common;
    pub mod large_bytes;
    pub mod mixed_bytes;
    pub mod short_bytes;
}

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
}
