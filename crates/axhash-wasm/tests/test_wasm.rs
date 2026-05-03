use axhash_wasm::{Hasher, axhash_seeded_wasm, axhash_wasm};
use wasm_bindgen_test::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_node_experimental);

#[wasm_bindgen_test]
fn test_basic_hashing() {
    let data = b"teknis-axhash";
    let res = axhash_wasm(data);

    assert!(res > 0);
}

#[wasm_bindgen_test]
fn test_seeded_consistency() {
    let data = b"axhash-consistency-test";
    let seed: u64 = 0xDEADBEEF;

    let h1 = axhash_seeded_wasm(data, seed);
    let h2 = axhash_seeded_wasm(data, seed);
    let h3 = axhash_seeded_wasm(data, 0);

    assert_eq!(h1, h2);
    assert_ne!(h1, h3);
}

#[wasm_bindgen_test]
fn test_streaming_hasher() {
    let mut hasher = Hasher::new(Some(0x12345678));
    hasher.update(b"part-1");
    hasher.update(b"part-2");

    let digest = hasher.digest();
    assert!(digest > 0);

    hasher.reset(0);
    hasher.update(b"part-1");
    hasher.update(b"part-2");
    assert_ne!(digest, hasher.digest());
}
