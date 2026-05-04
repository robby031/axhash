# axhash-wasm

`axhash-wasm` exposes AxHash for browsers, Node.js, and other WebAssembly runtimes through `wasm-bindgen`.

The npm package name is:

```text
axhash-wasm
```

## Build

```bash
wasm-pack build --target bundler --release
```

## Install

```bash
npm install axhash-wasm
```

## Examples

Hash bytes with the default seed:

```javascript
import { axhash } from "axhash-wasm";

const data = new TextEncoder().encode("hello axhash");
console.log(axhash(data));
```

Hash bytes with a custom seed:

```javascript
import { axhash_seeded } from "axhash-wasm";

const data = new TextEncoder().encode("hello axhash");
console.log(axhash_seeded(data, 0x12345678n));
```

Use the streaming hasher:

```javascript
import { Hasher } from "axhash-wasm";

const hasher = new Hasher(0x4444n);
hasher.update(new TextEncoder().encode("hello "));
hasher.update(new TextEncoder().encode("world"));
console.log(hasher.digest());
```

Reset and reuse the hasher:

```javascript
import { Hasher } from "axhash-wasm";

const hasher = new Hasher();
hasher.update(new Uint8Array([1, 2, 3]));
console.log(hasher.digest());

hasher.reset(7n);
hasher.update(new Uint8Array([4, 5, 6]));
console.log(hasher.digest());
```

Inspect runtime information:

```javascript
import { runtime_backend, runtime_has_aes, version } from "axhash-wasm";

console.log(runtime_backend());
console.log(runtime_has_aes());
console.log(version());
```

## Notes

- The returned digest is exposed as a JavaScript `BigInt`.
- Use `Uint8Array` for the input buffer.
- For large payloads, prefer repeated `Hasher.update(...)` calls.

## License

MIT.
