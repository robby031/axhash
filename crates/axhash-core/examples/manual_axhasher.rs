//! Contoh penggunaan manual AxHasher untuk advanced user
use axhash_core::AxHasher;
use core::hash::Hasher;

fn main() {
    let mut hasher = AxHasher::new_with_seed(0x2026);
    hasher.write_u64(0x1122_3344_5566_7788);
    hasher.write_u32(0x99aabbcc);
    hasher.write_u16(0xddee);
    hasher.write_u8(0xff);
    let hash = hasher.finish();
    println!("Manual AxHasher (custom seed): {hash:016x}");
}
