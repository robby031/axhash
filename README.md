`axhash (v0.1.1)`

axhash adalah hash function non-cryptographic untuk Rust yang dirancang untuk performa tinggi, distribusi bit yang merata, dan efisiensi pada workload nyata, khususnya pada key kecil hingga menengah dan penggunaan di struktur data seperti HashMap.

## Apa yang Baru di v0.1.1?

Versi ini membawa pembaruan cukup besar pada arsitektur internal untuk memaksimalkan efisiensi pada CPU modern, termasuk:

- Jalur NEON asimetris untuk ARMv8-A
- Akselerasi hardware AES `vaeseq_u8`, `vaesmcq_u8` untuk mixing lebih cepat
- Loop 128-byte yang mengoptimalkan pemrosesan data besar
- Tail processing tanpa branch untuk mengurangi misprediction
- Permutasi `Feistel Link` untuk memperkuat difusi bit dan ketahanan DoS

## Fitur Utama

- Kompatibel `no_std`, tanpa alokasi memori pada jalur utama
- Deterministik dan seeded, cocok untuk kebutuhan `indexing`, `cache`, dan `struktur data`
- Tidak bergantung pada crate eksternal
- Performa baik pada key kecil, menengah maupun payload besar
- Dirancang agar tahan terhadap collision attack pada workload umum `bukan kriptografi`

## Instalasi

Tambahkan ke Cargo.toml:

```toml
[dependencies]
axhash = "0.1.1"
```

## Benchmark (Apple M4)

Hasil benchmark internal menggunakan `Criterion.rs`:

![Hasil Hotloop](assets/screenshot_hotloop.png)
![Hasil Oneshoot](assets/screenshot_oneshoot.png)
![Hasil Streaming](assets/screenshot_streaming.png)

## Contoh Penggunaan

Hash bytes dengan seed:

```rust
use axhash::axhash_seeded;
let hash = axhash_seeded(b"axhash super power", 0x1234_5678);
println!("Hash: {hash:016x}");
```

HashMap dengan AxBuildHasher:

```rust
use axhash::AxBuildHasher;
use std::collections::HashMap;
let mut map = HashMap::with_hasher(AxBuildHasher::with_seed(0xDEADC0DE));
map.insert("key", "performance");
```

## Desain Singkat

axhash menggabungkan folded multiplication, branchless path untuk key kecil, dan teknik permutasi state untuk memastikan difusi bit yang merata dan performa konsisten pada berbagai ukuran data.

## Batasan

axhash bukan hash function kriptografi. `Jangan gunakan untuk password, tanda tangan digital, atau kebutuhan keamanan tingkat tinggi.`

## Lisensi

Proyek ini dirilis di bawah lisensi MIT.
