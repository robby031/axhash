# axhash-python

### Core Engine (Rust)

[![Crates.io](https://img.shields.io/crates/v/axhash-core?style=flat-square&color=orange&logo=rust)](https://crates.io/crates/axhash-core)
[![Documentation](https://img.shields.io/docsrs/axhash-core?style=flat-square&logo=docs.rs)](https://docs.rs/axhash-core)
[![Downloads](https://img.shields.io/crates/d/axhash-core?style=flat-square&color=darkgreen)](https://crates.io/crates/axhash-core)

### Extensions & Distribution

[![Python](https://img.shields.io/pypi/v/axhash-python?style=flat-square&logo=python&logoColor=white&color=blue)](https://pypi.org/project/axhash-python/)
[![FFI Downloads](https://img.shields.io/crates/d/axhash-ffi?style=flat-square&color=darkgreen)](https://crates.io/crates/axhash-ffi)
[![Support me](https://img.shields.io/badge/Support%20me-Ko--fi-F16061?style=flat-square&logo=ko-fi)](https://ko-fi.com/robby031)

Binding Python untuk engine AxHash, dibangun langsung di atas `axhash-core` menggunakan PyO3. Mudah digunakan untuk kebutuhan hash cepat di ekosistem Python.

---

## Instalasi

Build wheel (butuh Rust dan maturin):

```bash
maturin build --release
# atau untuk pengembangan
maturin develop
```

Install wheel hasil build:

```bash
pip install target/wheels/axhash_python-*.whl
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

```python
import axhash_python as axhash

# Hash langsung
print(axhash.axhash(b"hello"))
print(axhash.axhash_seeded(b"hello", 0x1234))

# Streaming hash
h = axhash.Hasher(seed=0x1234)
h.update(b"data1")
h.update(b"data2")
print(h.digest())

# Info runtime
print(axhash.runtime_backend())
print(axhash.runtime_has_aes())
```

---

## Lisensi

MIT. Bebas digunakan untuk open source maupun komersial.
