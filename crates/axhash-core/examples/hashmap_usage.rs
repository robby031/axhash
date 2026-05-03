use axhash_core::AxBuildHasher;
use std::collections::HashMap;

fn main() {
    // HashMap dengan seed custom
    let mut cache: HashMap<u64, &'static str, AxBuildHasher> =
        HashMap::with_hasher(AxBuildHasher::with_seed(0xfeed_beef));

    cache.insert(10, "pending");
    cache.insert(20, "confirmed");
    cache.insert(30, "finalized");

    println!("HashMap dengan seed custom:");
    for key in [10_u64, 20, 30] {
        println!("  key {key} => {:?}", cache.get(&key));
    }

    // HashMap dengan seed default
    let mut cache_default: HashMap<&str, i32, AxBuildHasher> =
        HashMap::with_hasher(AxBuildHasher::new());
    cache_default.insert("foo", 1);
    cache_default.insert("bar", 2);
    println!("\nHashMap dengan seed default:");
    for key in ["foo", "bar"] {
        println!("  key '{key}' => {:?}", cache_default.get(key));
    }
}
