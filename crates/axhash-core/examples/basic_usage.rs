use axhash_core::{AxHasher, axhash, axhash_of, axhash_of_seeded, axhash_seeded};
use core::hash::{Hash, Hasher};

#[derive(Hash)]
struct SessionKey {
    account_id: u64,
    region_id: u32,
    flags: u32,
}

fn main() {
    let seed = 0x1234_5678_9abc_def0;

    // Hash bytes dengan seed default
    let hash1 = axhash(b"hello axhash");
    println!("axhash (default seed):       {hash1:016x}");

    // Hash bytes dengan custom seed
    let hash2 = axhash_seeded(b"hello axhash", seed);
    println!("axhash_seeded:               {hash2:016x}");

    let key = SessionKey {
        account_id: 42,
        region_id: 7,
        flags: 3,
    };

    // Hash struct dengan seed default
    let hash3 = axhash_of(&key);
    println!("axhash_of (default seed):     {hash3:016x}");

    // Hash struct dengan custom seed
    let hash4 = axhash_of_seeded(&key, seed);
    println!("axhash_of_seeded:             {hash4:016x}");

    // Penggunaan manual AxHasher
    let mut hasher = AxHasher::new_with_seed(0x4444);
    hasher.write_u64(0x0102_0304_0506_0708);
    hasher.write_u32(0xaabb_ccdd);
    hasher.write_u16(0xeeff);
    hasher.write_u8(0x11);
    let manual_hash = hasher.finish();
    println!("Manual AxHasher:              {manual_hash:016x}");
}
