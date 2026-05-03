# axhash-core

### Core Engine (Rust)

[![Crates.io](https://img.shields.io/crates/v/axhash-core?style=flat-square&color=orange&logo=rust)](https://crates.io/crates/axhash-core)
[![Documentation](https://img.shields.io/docsrs/axhash-core?style=flat-square&logo=docs.rs)](https://docs.rs/axhash-core)
[![Downloads](https://img.shields.io/crates/d/axhash-core?style=flat-square&color=darkgreen)](https://crates.io/crates/axhash-core)

### Extensions & Distribution

[![Python](https://img.shields.io/pypi/v/axhash?style=flat-square&logo=python&logoColor=white&color=blue)](https://pypi.org/project/axhash-python/0.1.3/)
[![FFI Downloads](https://img.shields.io/crates/d/axhash-ffi?style=flat-square&color=darkgreen)](https://crates.io/crates/axhash-ffi)
[![Support me](https://img.shields.io/badge/Support%20me-Ko--fi-F16061?style=flat-square&logo=ko-fi)](https://ko-fi.com/robby031)

Mesin utama (core) AxHash: hash function non-cryptographic yang cepat, deterministik, dan fleksibel untuk Rust.

---

## Fitur Utama

- API Rust yang sederhana dan idiomatik
- Kompatibel `no_std` (bisa dipakai di embedded, kernel, dsb)
- Zero allocation di jalur utama
- Seeded/deterministik, cocok untuk struktur data, cache, indexing
- Backend otomatis memilih instruksi optimal (AES/NEON/dll)
- Bisa dipakai langsung oleh crate binding lain seperti `axhash-ffi`, `axhash-python`, dsb

---

## Instalasi

Tambahkan ke `Cargo.toml`:

```toml
[dependencies]
axhash-core = "0.1"
```

---

## Contoh Penggunaan

Hash bytes langsung:

```rust
use axhash_core::axhash_seeded;

let hash = axhash_seeded(b"hello world", 0x1234_5678);
println!("Hash: {hash:016x}");
```

Streaming hash (implementasi Hasher):

```rust
use axhash_core::AxHasher;
use std::hash::Hasher;

let mut hasher = AxHasher::with_seed(0x1234);
hasher.write(b"data");
let hash = hasher.finish();
```

---

## Lisensi

MIT. Bebas digunakan untuk open source maupun komersial.
