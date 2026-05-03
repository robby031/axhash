# axhash-wasm

Binding WebAssembly (WASM) untuk engine AxHash, dibangun langsung di atas `axhash-core` menggunakan `wasm-bindgen`. Memberikan performa hashing tingkat sistem langsung di browser maupun Node.js.

---

## Instalasi

Build paket WASM:

```bash
wasm-pack build --target bundler --release
```

Install NPM:

```bash
npm i axhash-rs-wasm
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

```javaScript
import {
    axhash,
    axhash_seeded,
    Hasher,
    runtime_backend,
    runtime_has_aes
} from 'axhash-rs-wasm';

async function run() {
    // Siapkan data sebagai Uint8Array
    const data = new TextEncoder().encode("hello");

    // 1. Hash Langsung
    // Mengembalikan BigInt untuk presisi u64 (Rust u64 -> JS BigInt)
    const hash = axhash(data);
    console.log("Simple Hash:", hash); // Contoh: 15188980152138703324n

    // 2. Hash dengan Seed
    const seeded = axhash_seeded(data, 0x1234n);
    console.log("Seeded Hash:", seeded);

    // 3. Streaming Hash (Stateful)
    // Cocok untuk memproses data besar dalam potongan-potongan (chunks)
    const hasher = new Hasher(0x1234n);
    hasher.update(new Uint8Array([1, 2, 3]));
    hasher.update(new Uint8Array([4, 5, 6]));
    console.log("Streaming Digest:", hasher.digest());

    // 4. Informasi Runtime
    console.log("Backend yang digunakan:", runtime_backend()); // 'aes-hardware' atau 'scalar'
    console.log("Dukungan AES-NI Hardware:", runtime_has_aes());
}

run().catch(console.error);
```

---

## Lisensi

MIT. Bebas digunakan untuk open source maupun komersial.
