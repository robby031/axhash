use axhash_core::{
    AxHasher, RuntimeBackend, axhash, axhash_seeded, runtime_backend, runtime_has_aes,
};
use core::hash::Hasher as _;
use wasm_bindgen::prelude::*;
use std::cell::RefCell;

const BUF_SIZE: usize = 64 * 1024;
thread_local! {
       static INTERNAL_BUF: RefCell<Vec<u8>> = RefCell::new(vec![0u8; BUF_SIZE]);
}

fn backend_name(backend: RuntimeBackend) -> &'static str {
    match backend {
        RuntimeBackend::Scalar => "scalar",
        RuntimeBackend::Aarch64AesNeon => "aarch64-aes-neon",
        RuntimeBackend::X86_64AesAvx2 => "x86_64-aes-avx2",
    }
}

#[wasm_bindgen(js_name = axhash)]
pub fn axhash_wasm(data: &[u8]) -> u64 {
    axhash(data)
}

#[wasm_bindgen(js_name = axhash_seeded)]
pub fn axhash_seeded_wasm(data: &[u8], seed: u64) -> u64 {
    axhash_seeded(data, seed)
}

#[wasm_bindgen(js_name = runtime_backend)]
pub fn runtime_backend_wasm() -> String {
    backend_name(runtime_backend()).to_string()
}

#[wasm_bindgen(js_name = runtime_has_aes)]
pub fn runtime_has_aes_wasm() -> bool {
    runtime_has_aes()
}

#[wasm_bindgen]
pub struct Hasher {
    inner: AxHasher,
    buf: Vec<u8>,
}

#[wasm_bindgen]
impl Hasher {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: Option<u64>) -> Self {
        Self {
            inner: AxHasher::new_with_seed(seed.unwrap_or(0)),
            buf: vec![0u8; BUF_SIZE],
        }
    }

    pub fn reset(&mut self, seed: u64) {
        self.inner = AxHasher::new_with_seed(seed);
        self.buf.fill(0);
    }

    pub fn update(&mut self, data: &[u8]) {
        let mut offset = 0;
        let len = data.len();
        while offset < len {
            let end = usize::min(offset + BUF_SIZE, len);
            let chunk = &data[offset..end];

            if chunk.len() < BUF_SIZE {
                self.buf[..chunk.len()].copy_from_slice(chunk);
                self.inner.write(&self.buf[..chunk.len()]);
            } else {
                self.inner.write(chunk);
            }

            offset += chunk.len();
        }
    }

    pub fn digest(&self) -> u64 {
        self.inner.finish()
    }
}

#[wasm_bindgen(js_name = version)]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
