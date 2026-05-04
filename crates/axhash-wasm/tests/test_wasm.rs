use axhash_wasm::{Hasher, axhash_seeded_wasm, axhash_wasm};
use wasm_bindgen_test::*;
use web_sys::console;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_node_experimental);
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_basic_hashing() {
    let data = b"teknis-axhash";
    let res = axhash_wasm(data);
    console::log_1(&format!("Hash result: {res}").into());
    assert!(res > 0);
}

#[wasm_bindgen_test]
fn test_seeded_consistency() {
    let data = b"axhash-consistency-test";
    let seed: u64 = 0xDEADBEEF;

    let h1 = axhash_seeded_wasm(data, seed);
    let h2 = axhash_seeded_wasm(data, seed);
    let h3 = axhash_seeded_wasm(data, 0);

    let result = format!("h1: {h1}, h2: {h2}, h3: {h3}");
    console::log_1(&result.into());

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
