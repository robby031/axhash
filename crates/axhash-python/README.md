# axhash-python

`axhash-python` provides Python bindings for AxHash through `PyO3`.

The package is installed as `axhash-python`, but the import name is simply:

```python
import axhash
```

## Build Or Install

Build a wheel locally:

```bash
maturin build --release
```

Install in development mode:

```bash
maturin develop
```

## Examples

Hash bytes with the default seed:

```python
import axhash

print(axhash.axhash(b"hello axhash"))
```

Hash bytes with a custom seed:

```python
import axhash

print(axhash.axhash_seeded(b"hello axhash", 0x12345678))
```

Use the streaming hasher:

```python
import axhash

h = axhash.Hasher(seed=0x4444)
h.update(b"hello ")
h.update(b"world")
print(h.digest())
```

Reset and reuse the hasher:

```python
import axhash

h = axhash.Hasher()
h.update(b"first payload")
print(h.digest())

h.reset(7)
h.update(b"second payload")
print(h.digest())
```

Inspect runtime capabilities:

```python
import axhash

print(axhash.runtime_backend())
print(axhash.runtime_has_aes())
```

## API

- `axhash(data: bytes) -> int`
- `axhash_seeded(data: bytes, seed: int) -> int`
- `Hasher(seed: int = 0)`
- `Hasher.update(data: bytes) -> None`
- `Hasher.digest() -> int`
- `Hasher.reset(seed: int) -> None`
- `runtime_backend() -> str`
- `runtime_has_aes() -> bool`

## License

MIT.
