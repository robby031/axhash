# axhash-python

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
