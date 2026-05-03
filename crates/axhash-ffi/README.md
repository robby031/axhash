# axhash-ffi

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
