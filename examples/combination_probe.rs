use axhash::axhash_seeded;
use std::collections::HashMap;

fn main() {
    let mut by_len: HashMap<usize, (usize, usize)> = HashMap::new();

    for len in 2..=20 {
        let mut counts: HashMap<u64, Vec<(usize, u8)>> = HashMap::new();
        for pos in 0..len {
            for v in 1u32..=255 {
                let mut key = vec![0u8; len];
                key[pos] = v as u8;
                let h = axhash_seeded(&key, 0);
                counts.entry(h).or_default().push((pos, v as u8));
            }
        }
        let total: usize = counts.values().map(|v| v.len()).sum();
        let coll: usize = counts.values().map(|v| v.len() - 1).sum();
        by_len.insert(len, (total, coll));
    }

    println!("TwoBytes 1 non zero byte per length:");
    let mut entries: Vec<_> = by_len.iter().collect();
    entries.sort_by_key(|(l, _)| *l);
    for (len, (total, coll)) in entries {
        let pct = 100.0 * (*coll as f64) / (*total as f64);
        if *coll > 0 {
            println!(
                "  len={:>2}: keys={} coll={} ({:.3}%)",
                len, total, coll, pct
            );
        }
    }

    let mut pool: HashMap<u64, Vec<(usize, usize, u8)>> = HashMap::new();
    let mut total_keys = 0usize;
    for len in 2..=20 {
        for pos in 0..len {
            for v in 1u32..=255 {
                let mut key = vec![0u8; len];
                key[pos] = v as u8;
                let h = axhash_seeded(&key, 0);
                pool.entry(h).or_default().push((len, pos, v as u8));
                total_keys += 1;
            }
        }
    }
    let coll: usize = pool.values().map(|v| v.len() - 1).sum();
    println!("\ntotal={} uniq={} coll={}", total_keys, pool.len(), coll);

    println!("\nExample collisions cross-length:");
    let mut shown = 0;
    let mut large: Vec<_> = pool.values().filter(|v| v.len() > 1).collect();
    large.sort_by_key(|v| std::cmp::Reverse(v.len()));
    for v in large.iter().take(5) {
        println!(
            "  count={}: {:?}",
            v.len(),
            v.iter().take(6).collect::<Vec<_>>()
        );
        shown += 1;
        if shown == 5 {
            break;
        }
    }
}
