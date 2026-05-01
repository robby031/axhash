# axhash

`axhash` adalah hash function non-cryptographic untuk Rust yang dioptimalkan untuk:

- workload `HashMap` dengan key kecil
- penggunaan `no_std`
- hashing bytes mentah dengan seed deterministik

Crate ini menyediakan:

- `axhash_seeded(&[u8], u64) -> u64`
- `axhash_of_seeded<T: Hash>(&T, u64) -> u64`
- `AxHasher` untuk implementasi `core::hash::Hasher`
- `AxBuildHasher` untuk dipakai langsung di `HashMap`

## Karakteristik

- `#![no_std]`
- seeded / deterministic
- zero allocation di jalur hashing inti
- dioptimalkan untuk primitive writes dan key kecil
- cocok untuk struktur data in-memory, indexing, cache key, dan workload lookup cepat

## Bukan Untuk

`axhash` bukan hash cryptographic. Jangan gunakan untuk:

- password hashing
- tanda tangan digital
- MAC / authentication
- penyimpanan hash yang butuh jaminan keamanan kriptografis

## Instalasi

```toml
[dependencies]
axhash = "0.1.0"
```

## Contoh Cepat

Hash raw bytes:

```rust
use axhash::axhash_seeded;

let hash = axhash_seeded(b"hello world", 0x1234_5678);
println!("{hash:016x}");
```

Hash tipe yang mengimplementasikan `Hash`:

```rust
use axhash::axhash_of_seeded;
use core::hash::Hash;

#[derive(Hash)]
struct UserKey {
    id: u64,
    shard: u32,
}

let key = UserKey { id: 42, shard: 7 };
let hash = axhash_of_seeded(&key, 0xdead_beef);
println!("{hash:016x}");
```

Pakai di `HashMap`:

```rust
use axhash::AxBuildHasher;
use std::collections::HashMap;

let mut map: HashMap<u64, &str, AxBuildHasher> =
    HashMap::with_hasher(AxBuildHasher::with_seed(0x1234));

map.insert(1, "one");
map.insert(2, "two");

assert_eq!(map.get(&1), Some(&"one"));
```

## Menjalankan Example

Example siap pakai ada di folder `examples/`:

```bash
cargo run --example basic_usage
cargo run --example hashmap_usage
```

## Benchmark

Repo ini menyertakan dua benchmark:

```bash
cargo bench --bench stress_test
cargo bench --bench head_to_head
```

`stress_test` fokus ke profil internal `axhash`, sedangkan `head_to_head` membandingkan `axhash` melawan `ahash`, `foldhash`, `rapidhash`, dan `wyhash`.

## Desain Singkat

`axhash` memakai kombinasi:

- folded multiplication (`u64 x u64 -> u128 -> xor-fold`)
- short-path khusus untuk byte slice kecil
- sponge ringan untuk jalur `Hasher` berbasis primitive writes
- `BuildHasher` murah untuk workload `HashMap`

Tujuan utamanya adalah memberi performa kuat pada workload nyata, terutama key kecil dan operasi lookup/insert intensif.

## Lisensi

Project ini dirilis dengan lisensi MIT. Lihat [LICENSE-MIT](LICENSE-MIT).
