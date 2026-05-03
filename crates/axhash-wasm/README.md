# axhash-wasm

### Core Engine (Rust)

[![Crates.io](https://img.shields.io/crates/v/axhash-core?style=flat-square&color=orange&logo=rust)](https://crates.io/crates/axhash-core)
[![Documentation](https://img.shields.io/docsrs/axhash-core?style=flat-square&logo=docs.rs)](https://docs.rs/axhash-core)
[![Downloads](https://img.shields.io/crates/d/axhash-core?style=flat-square&color=darkgreen)](https://crates.io/crates/axhash-core)

### Extensions & Distribution

[![Python](https://img.shields.io/pypi/v/axhash-python?style=flat-square&logo=python&logoColor=white&color=blue)](https://pypi.org/project/axhash-python/)
[![FFI Downloads](https://img.shields.io/crates/d/axhash-ffi?style=flat-square&color=darkgreen)](https://crates.io/crates/axhash-ffi)
[![Support me](https://img.shields.io/badge/Support%20me-Ko--fi-F16061?style=flat-square&logo=ko-fi)](https://ko-fi.com/robby031)

Binding WebAssembly (WASM) untuk engine AxHash, dibangun langsung di atas `axhash-core` menggunakan `wasm-bindgen`. Memberikan performa hashing tingkat sistem langsung di browser maupun Node.js.

---

## Instalasi

Build paket WASM:

```bash
wasm-pack build --target bundler --release
```

Install NPM:

```bash
npm i axhash-rs-wasm
```

---

## API Utama

- `axhash(data: bytes) -> int` — Hash cepat tanpa seed
- `axhash_seeded(data: bytes, seed: int) -> int` — Hash dengan seed custom
- `runtime_backend() -> str` — Info backend yang dipakai
- `runtime_has_aes() -> bool` — Deteksi akselerasi AES
- `Hasher(seed: int = 0)` — Streaming hash (update/incremental)

---

## Contoh Penggunaan

```javaScript
import init, { axhash, axhash_seeded, Hasher, runtime_backend, runtime_has_aes } from 'axhash-rs-wasm';

async function run() {
    // Inisialisasi modul WASM
    await init();

    // Siapkan data sebagai Uint8Array
    const encoder = new TextEncoder();
    const data = encoder.encode("hello");

    // Hash langsung (Mengembalikan BigInt untuk presisi u64)
    console.log("Simple Hash:", axhash(data));
    console.log("Seeded Hash:", axhash_seeded(data, 0x1234n));

    // Streaming hash (Stateful)
    const h = new Hasher(0x1234n);
    h.update(new Uint8Array([1, 2, 3]));
    h.update(new Uint8Array([4, 5, 6]));
    console.log("Streaming Digest:", h.digest());

    // Info runtime
    console.log("Backend:", runtime_backend());
    console.log("Has AES Acceleration:", runtime_has_aes());
}

run();
```

---

## Lisensi

MIT. Bebas digunakan untuk open source maupun komersial.
