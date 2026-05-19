use axhash::axhash_seeded;

fn main() {
    for &len in &[5usize, 6, 7, 8] {
        let target_bit = (len * 8 - 1) as u32;
        println!("=== len={} target input bit={} ===", len, target_bit);
        let mut diff_pattern = 0u64;
        let mut consistent = true;
        let mut first_diff = 0u64;
        for trial in 0u64..20 {
            let mut key = [0u8; 16];
            // semi random key
            let mut seed_state = trial
                .wrapping_mul(0x9E3779B97F4A7C15)
                .wrapping_add(0xDEADBEEF);
            for b in key.iter_mut().take(len) {
                seed_state = seed_state.wrapping_mul(0xBF58476D1CE4E5B9) ^ (seed_state >> 31);
                *b = (seed_state & 0xff) as u8;
            }
            let h1 = axhash_seeded(&key[..len], trial);
            let byte_idx = (target_bit / 8) as usize;
            let bit_idx = target_bit % 8;
            key[byte_idx] ^= 1u8 << bit_idx;
            let h2 = axhash_seeded(&key[..len], trial);
            let diff = h1 ^ h2;
            if trial == 0 {
                first_diff = diff;
            } else if diff != first_diff {
                consistent = false;
            }
            diff_pattern |= diff;
            if trial < 4 {
                println!(
                    "  trial {}: h1={:016x} h2={:016x} diff={:016x}",
                    trial, h1, h2, diff
                );
            }
        }
        println!(
            "  pengabungan dari 20 percobaan lebih: {:016x}",
            diff_pattern
        );
        println!("  Perbedaan identik: {}", consistent);
    }
}
