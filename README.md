# AxHash

### Core Engine (Rust)

[![Crates.io](https://img.shields.io/crates/v/axhash-core?style=flat-square&color=orange&logo=rust)](https://crates.io/crates/axhash-core)
[![Documentation](https://img.shields.io/docsrs/axhash-core?style=flat-square&logo=docs.rs)](https://docs.rs/axhash-core)
[![Downloads](https://img.shields.io/crates/d/axhash-core?style=flat-square&color=darkgreen)](https://crates.io/crates/axhash-core)

### Extensions & Distribution

[![Python](https://img.shields.io/pypi/v/axhash-python?style=flat-square&logo=python&logoColor=white&color=blue)](https://pypi.org/project/axhash-python/)
[![FFI Downloads](https://img.shields.io/crates/d/axhash-ffi?style=flat-square&color=darkgreen)](https://crates.io/crates/axhash-ffi)
[![Support me](https://img.shields.io/badge/Support%20me-Ko--fi-F16061?style=flat-square&logo=ko-fi)](https://ko-fi.com/robby031)

Keluarga hash function modern untuk Rust, C, Wasm, dan Python. Fokus pada performa, portabilitas, dan kemudahan integrasi.

---

## Daftar Crate

- **axhash-core**  
  Mesin utama hashing, API Rust, dan kompatibilitas `no_std`.
- **axhash-ffi**  
  Lapisan FFI stabil untuk C/C++ dan bahasa lain, menghasilkan `staticlib`/`cdylib` dan header C otomatis.
- **axhash-python**  
  Binding Python berbasis PyO3, siap dipakai di ekosistem Python modern.
- **axhash-wasm**  
  Binding Wasm berbasis `wasm-bindgen`, siap dipakai di ekosistem Wasm modern.

---
Documentasi
- [Doc Axhash-core](crates/axhash-core/README.md)
- [Doc Axhash-ffi](crates/axhash-ffi/README.md)
- [Doc Axhash-python](crates/axhash-python/README.md)
- [Doc Axhash-wasm](crates/axhash-wasm/README.md)
---

## Benchmark internal menggunakan `Criterion.rs`:

![Hasil Hotloop](assets/screenshot_hotloop.png)
![Hasil Oneshoot](assets/screenshot_oneshoot.png)
![Hasil Streaming](assets/screenshot_streaming.png)

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
- `crates/axhash-wasm` — binding WebAssembly
---

## Kontribusi

Kontribusi sangat terbuka! Silakan buka issue, pull request, atau diskusi jika ada ide, bug, atau kebutuhan integrasi lintas bahasa.

---

## Lisensi

MIT. Silakan gunakan untuk kebutuhan open source maupun komersial.
