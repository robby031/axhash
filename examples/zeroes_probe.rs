use axhash::axhash_seeded;
use std::collections::HashMap;

fn main() {
    // SeedBlockOffset-style: 1 non-zero byte at varying offset for keys of varying length.
    let mut pool: HashMap<u64, Vec<(usize, usize, u8)>> = HashMap::new();
    let mut total = 0usize;
    for len in 8..=16 {
        for pos in 0..len {
            for v in 1u32..=255 {
                let mut k = vec![0u8; len];
                k[pos] = v as u8;
                let h = axhash_seeded(&k, 0);
                pool.entry(h).or_default().push((len, pos, v as u8));
                total += 1;
            }
        }
    }
    let coll: usize = pool.values().map(|v| v.len() - 1).sum();
    println!("Pool len 8..=16, 1 non-zero byte: total={} unique={} coll={}", total, pool.len(), coll);
    let mut large: Vec<_> = pool.values().filter(|v| v.len() > 1).collect();
    large.sort_by_key(|v| std::cmp::Reverse(v.len()));
    println!("\nTop colliding groups:");
    for g in large.iter().take(6) {
        println!("  count={}: {:?}", g.len(), g.iter().take(5).collect::<Vec<_>>());
    }
}
