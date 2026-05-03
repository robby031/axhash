# axhash-ffi

### Core Engine (Rust)

[![Crates.io](https://img.shields.io/crates/v/axhash-core?style=flat-square&color=orange&logo=rust)](https://crates.io/crates/axhash-core)
[![Documentation](https://img.shields.io/docsrs/axhash-core?style=flat-square&logo=docs.rs)](https://docs.rs/axhash-core)
[![Downloads](https://img.shields.io/crates/d/axhash-core?style=flat-square&color=darkgreen)](https://crates.io/crates/axhash-core)

### Extensions & Distribution

[![Python](https://img.shields.io/pypi/v/axhash?style=flat-square&logo=python&logoColor=white&color=blue)](https://pypi.org/project/axhash-python/0.1.3/)
[![FFI Downloads](https://img.shields.io/crates/d/axhash-ffi?style=flat-square&color=darkgreen)](https://crates.io/crates/axhash-ffi)
[![Support me](https://img.shields.io/badge/Support%20me-Ko--fi-F16061?style=flat-square&logo=ko-fi)](https://ko-fi.com/robby031)

Lapisan FFI stabil untuk AxHash, menjembatani `axhash-core` dengan C/C++ dan bahasa lain.

---

## Fitur Utama

- Mengekspor simbol `extern "C"` yang stabil
- Menghasilkan library `staticlib` dan `cdylib` (bisa dipakai di C/C++, Go, Zig, Swift, Kotlin, dsb)
- Header C otomatis (`include/axhash.h`) via cbindgen
- ABI stabil, mudah diintegrasikan ke berbagai bahasa

---

## Build

Build library dan header:

```bash
cargo build -p axhash-ffi --release
```

Header C akan tersedia di:

```
crates/axhash-ffi/include/axhash.h
```

---

## Contoh Penggunaan (C)

```c
#include "axhash.h"

uint64_t hash = axhash_seeded((const uint8_t*)"hello", 5, 0x1234);
```

---

## Integrasi

Library ini bisa dijadikan dasar binding ke C++, Python (ctypes/cffi), Go (cgo), Zig, Swift, Kotlin Native, dan lain-lain.

---

## Lisensi

MIT. Bebas digunakan untuk open source maupun komersial.
