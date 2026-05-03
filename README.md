# AxHash

### Core Engine (Rust)

[![Crates.io](https://img.shields.io/crates/v/axhash-core?style=flat-square&color=orange&logo=rust)](https://crates.io/crates/axhash-core)
[![Documentation](https://img.shields.io/docsrs/axhash-core?style=flat-square&logo=docs.rs)](https://docs.rs/axhash-core)
[![Downloads](https://img.shields.io/crates/d/axhash-core?style=flat-square&color=darkgreen)](https://crates.io/crates/axhash-core)

### Extensions & Distribution

[![Python](https://img.shields.io/pypi/v/axhash-python?style=flat-square&logo=python&logoColor=white&color=blue)](https://pypi.org/project/axhash-python/)
[![FFI Downloads](https://img.shields.io/crates/d/axhash-ffi?style=flat-square&color=darkgreen)](https://crates.io/crates/axhash-ffi)
[![Support me](https://img.shields.io/badge/Support%20me-Ko--fi-F16061?style=flat-square&logo=ko-fi)](https://ko-fi.com/robby031)

Keluarga hash function modern untuk Rust, C, dan Python. Fokus pada performa, portabilitas, dan kemudahan integrasi.

---

## Daftar Crate

- **axhash-core**  
  Mesin utama hashing, API Rust, dan kompatibilitas `no_std`.
- **axhash-ffi**  
  Lapisan FFI stabil untuk C/C++ dan bahasa lain, menghasilkan `staticlib`/`cdylib` dan header C otomatis.
- **axhash-python**  
  Binding Python berbasis PyO3, siap dipakai di ekosistem Python modern.

---

## Instalasi & Penggunaan

### 1. Rust (`axhash-core`)

Tambahkan ke `Cargo.toml`:

```toml
[dependencies]
axhash-core = "0.1"
```

Contoh penggunaan:

```rust
use axhash_core::axhash_seeded;

let hash = axhash_seeded(b"hello world", 0x1234_5678);
println!("{hash:016x}");
```

Untuk streaming hash:

```rust
use axhash_core::AxHasher;
use std::hash::Hasher;

let mut hasher = AxHasher::with_seed(0x1234);
hasher.write(b"data");
let hash = hasher.finish();
```

Jalankan test:

```bash
cargo test -p axhash-core
```

---

### 2. C/C++ dan FFI (`axhash-ffi`)

Build library dan header:

```bash
cargo build -p axhash-ffi --release
```

Header C akan tersedia di:

```
crates/axhash-ffi/include/axhash.h
```

Contoh penggunaan di C:

```c
#include "axhash.h"

uint64_t hash = axhash_seeded((const uint8_t*)"hello", 5, 0x1234);
```

---

### 3. Python (`axhash-python`)

Build wheel Python:

```bash
cd crates/axhash-python
maturin build --release
# atau
maturin develop
```

Instalasi wheel:

```bash
pip install target/wheels/axhash_python-*.whl
```

Contoh penggunaan:

```python
import axhash_python as axhash

print(axhash.axhash(b"hello"))
print(axhash.axhash_seeded(b"hello", 0x1234))

h = axhash.Hasher(seed=0x1234)
h.update(b"data")
print(h.digest())
```

---

## Benchmark internal menggunakan `Criterion.rs`:

![Hasil Hotloop](assets/screenshot_hotloop.png)
![Hasil Oneshoot](assets/screenshot_oneshoot.png)
![Hasil Streaming](assets/screenshot_streaming.png)

---

---

## CI & Rilis

- Semua crate diuji otomatis di CI (`.github/workflows/ci.yml`)
- Rilis artefak native dan wheel Python otomatis saat tag baru (`.github/workflows/release.yml`)
- Script `scripts/package-release.py` membundel header, lisensi, README, dan library ke archive siap distribusi

---

## Struktur Workspace

- `crates/axhash-core` — engine utama, API Rust, dan backend
- `crates/axhash-ffi` — FFI dan distribusi native
- `crates/axhash-python` — binding Python

---

## Kontribusi

Kontribusi sangat terbuka! Silakan buka issue, pull request, atau diskusi jika ada ide, bug, atau kebutuhan integrasi lintas bahasa.

---

## Lisensi

MIT. Silakan gunakan untuk kebutuhan open source maupun komersial.
