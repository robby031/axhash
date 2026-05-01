use axhash::AxBuildHasher;
use std::collections::HashMap;

fn main() {
    let mut cache: HashMap<u64, &'static str, AxBuildHasher> =
        HashMap::with_hasher(AxBuildHasher::with_seed(0xfeed_beef));

    cache.insert(10, "pending");
    cache.insert(20, "confirmed");
    cache.insert(30, "finalized");

    for key in [10_u64, 20, 30] {
        println!("key {key} => {:?}", cache.get(&key));
    }
}
