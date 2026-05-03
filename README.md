# axhash workspace

Workspace ini sekarang dipisah menjadi dua crate agar core hashing tetap bersih dan distribusi lintas bahasa tidak mencemari jalur `no_std`.

## Crates

- `crates/axhash-core`: engine hashing utama, API Rust, dispatch backend, dan kompatibilitas `no_std`
- `crates/axhash-ffi`: wrapper `extern "C"`, opaque pointer, `cbindgen`, dan artefak `staticlib` / `cdylib`
- `crates/axhash-python`: binding Python pertama berbasis `PyO3` langsung di atas `axhash-core`

## Build

Core Rust:

```bash
cargo test -p axhash-core
```

FFI / distribusi native:

```bash
cargo build -p axhash-ffi --release
```

Python wheel:

```bash
cd crates/axhash-python
maturin build --release
```

## CI dan Release

- `.github/workflows/ci.yml` menguji workspace, memvalidasi `no_std` untuk `axhash-core`, membangun artefak native lintas target, dan membangun wheel Python
- `.github/workflows/release.yml` mengunggah archive native dan wheel Python saat tag `v*` dibuat
- `scripts/package-release.py` membundel header C, lisensi, README, dan library hasil build menjadi archive siap distribusi

Header C yang digenerate tersedia di `crates/axhash-ffi/include/axhash.h`.
