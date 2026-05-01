use axhash::{axhash_of_seeded, axhash_seeded};
use core::hash::Hash;

#[derive(Hash)]
struct SessionKey {
    account_id: u64,
    region_id: u32,
    flags: u32,
}

fn main() {
    let seed = 0x1234_5678_9abc_def0;

    let bytes_hash = axhash_seeded(b"hello axhash", seed);
    println!("bytes hash   : {bytes_hash:016x}");

    let key = SessionKey {
        account_id: 42,
        region_id: 7,
        flags: 3,
    };

    let struct_hash = axhash_of_seeded(&key, seed);
    println!("struct hash  : {struct_hash:016x}");
}
