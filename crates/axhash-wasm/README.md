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

## Contoh Penggunaan & Best Practice

```javascript
import {
  axhash,
  axhash_seeded,
  Hasher,
  runtime_backend,
  runtime_has_aes,
} from "axhash-rs-wasm";

async function run() {
  // ---
  // 1. Siapkan data sebagai Uint8Array (Best Practice)
  // ---
  // Untuk performa maksimal, pastikan data yang dikirim ke WASM adalah Uint8Array
  // yang view-nya langsung ke WASM memory (bukan hasil copy dari JS heap)
  // Contoh paling aman:
  const data = new TextEncoder().encode("hello");

  // ---
  // 2. Hash Langsung
  // ---
  // Fungsi axhash menerima Uint8Array, return BigInt (u64 presisi penuh)
  const hash = axhash(data);
  console.log("Simple Hash:", hash); // Contoh: 15188980152138703324n

  // ---
  // 3. Hash dengan Seed
  // ---
  const seeded = axhash_seeded(data, 0x1234n);
  console.log("Seeded Hash:", seeded);

  // ---
  // 4. Streaming Hash (Stateful)
  // ---
  // Untuk data besar, proses bertahap (chunked)
  const hasher = new Hasher(0x1234n);
  // Contoh update bertahap
  hasher.update(new Uint8Array([1, 2, 3]));
  hasher.update(new Uint8Array([4, 5, 6]));
  console.log("Streaming Digest:", hasher.digest());

  // ---
  // 5. Informasi Runtime
  // ---
  console.log("Backend yang digunakan:", runtime_backend()); // 'aes-hardware' atau 'scalar'
  console.log("Dukungan AES-NI Hardware:", runtime_has_aes());

  // ---
  // 6. Zero-Copy Buffer (Advanced)
  // ---
  // Untuk performa maksimal (misal data besar dari fetch/file):
  // - Alokasikan buffer di WASM memory (misal via WebAssembly.Memory)
  // - Isi data langsung ke buffer tersebut pakai Uint8Array
  // - Kirim view-nya ke axhash/update
  //
  // Contoh:
  // const wasmMemory = instance.exports.memory;
  // const ptr = ... // offset buffer di WASM
  // const len = ... // panjang data
  // const view = new Uint8Array(wasmMemory.buffer, ptr, len);
  // ...isi view dengan data...
  // axhash(view);
  //
}

run().catch(console.error);
```

---

## Tips Performa Maksimal

- Selalu gunakan `Uint8Array` yang view-nya ke WASM memory untuk data besar.
- Hindari TextEncoder/TextDecoder jika data sudah dalam bentuk bytes.
- Untuk streaming/batch, gunakan `Hasher.update()` berkali-kali, lalu `digest()`.
- Return value hash adalah `BigInt` (presisi penuh, sesuai Rust u64).
- Untuk interoperabilitas dengan sistem lain, bisa split BigInt ke dua u32:

```javascript
function u64ToPair(u64) {
  return [Number(u64 & 0xffffffffn), Number(u64 >> 32n)];
}
```

---

---

## Lisensi

MIT. Bebas digunakan untuk open source maupun komersial.
