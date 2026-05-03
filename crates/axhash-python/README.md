# axhash-python

Binding Python pertama untuk engine AxHash, dibangun langsung di atas `axhash-core` menggunakan `PyO3`.

## API

- `axhash(data: bytes) -> int`
- `axhash_seeded(data: bytes, seed: int) -> int`
- `runtime_backend() -> str`
- `runtime_has_aes() -> bool`
- `Hasher(seed: int = 0)` untuk mode streaming

## Build Wheel

```bash
maturin build --release
```
