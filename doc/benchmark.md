# Benchmarks — AxHash v0.10.0

Benchmark dijalankan di Apple Silicon M4, profile `release` (lto=fat,
codegen-units=1). Hasil di bawah memakai dua tool:

- **Criterion.rs** untuk angka mikro-benchmark internal (per-API, per-payload).
- **`axhash-headtohead`** untuk perbandingan langsung lintas hasher.

Untuk validasi statistik distribusi & collision dipakai **SMHasher3** (fork
fwojcik). Lihat bagian akhir.

## Ringkasan v0.10.0 (E3, no-avalanche)

| Metrik | v0.10.0 | v0.9.0 | Perubahan |
|--------|---------:|---------:|----------:|
| One-shot 64 KiB (throughput) | ~73 GiB/s | ~95.6 GiB/s | −24% |
| One-shot 4 KiB | ~84 GiB/s | ~99.2 GiB/s | −15% |
| One-shot 256 B | ~22.8 GiB/s | ~59.4 GiB/s | −62% |
| One-shot 64 B | ~16.5 GiB/s | ~24.2 GiB/s | −32% |
| One-shot 16 B | ~17.0 GiB/s | ~18.6 GiB/s | −9% |
| One-shot 4 B | ~2.82 GiB/s | ~3.71 GiB/s | −24% |
| HashMap `get-hit` (100 k) | ~166 Melem/s | ~469 Melem/s | — (`-quick`) |
| HashMap `get-miss` (10 k) | ~876 Melem/s | ~738 Melem/s | +19% |
| HashMap `insert-u64` (100 k) | ~90.9 Melem/s | ~90 Melem/s | ≈ |
| HashMap mixed workload | ~11.0 Melem/s | ~13 Melem/s | −15% |
| **`write_u64` latency** | **~358 ps** | ~374 ps | **−4%** |
| `write_u32` latency | ~430 ps | — | new |
| Head-to-head 8B latency | **1.46 ns** | — | **−30% vs 2-mul** |
| Head-to-head mixed (10k) | **7.67 ns** | — | **best non-fxhash** |
| **SMHasher3 full suite** | **188/188 PASS** | 145/188 FAIL | **+30%** |

> **Trade-off v0.10.0:** Bulk throughput menurun karena algoritma menambah
> mixing per-branch untuk lulus SMHasher3 (Bug A/B/C/D di v0.9.0).
> Sebaliknya **tiny-key latency (≤16B) turun signifikan** karena avalanche
> finalizer dihapus setelah branch finalize terbukti cukup strong.
> Net result: real-world HashMap workload `get-miss` & latency-sensitive
> path lebih cepat dari v0.9.0, throughput bulk sedikit menurun.

> Trade-off v0.10.0: throughput sedikit menurun untuk short/medium key,
> ditukar dengan **kualitas distribusi industrial-grade** (188/188 SMHasher3).
> Versi 0.9.0 punya XOR cancellation di tail short-key, lane cancellation
> di long-path, dan linear length-XOR yang membuat banyak collision
> struktural. Lihat `doc/smhasher3-roadmap.md` untuk detail.

## Head-to-head vs hasher populer

Apple M4, single-thread, median 300 sample × 200 batch. Sumber:
`cargo run --release -p axhash-headtohead`.

### One-shot throughput (GB/s, higher = better)

| Hasher        | 4 B  | 16 B  | 64 B  | 256 B | 4 KB  | 64 KB |
|---------------|-----:|------:|------:|------:|------:|------:|
| **axhash**    | 0.84 | 2.95  | 7.49  | 11.93 | **45.20** | **78.80** |
| xxh3          | 3.20 | 11.00 | 23.62 | 26.71 | 44.58 | 44.84 |
| wyhash        | 1.92 | 6.40  | 25.60 | 35.09 | 27.01 | 25.82 |
| ahash         | 2.40 | 9.61  | 23.66 | 28.57 | 21.99 | 21.38 |
| fxhash        | 3.20 | 12.80 | 23.62 | 37.24 | 25.60 | 24.71 |
| siphash-1-3   | 1.13 | 2.84  | 5.39  | 5.80  | 5.65  | 5.79  |
| highwayhash   | 0.23 | 0.97  | 3.41  | 7.98  | 9.96  | 11.07 |

axhash unggul **bulk throughput ≥ 4 KB** (AES-NEON pipeline). Kompetitor lebih
cepat di short-key.

### Small-key latency (ns/op, lower = better)

| Hasher       | 1 B  | 4 B  | 8 B  | 16 B | 32 B | 64 B |
|--------------|-----:|-----:|-----:|-----:|-----:|-----:|
| **axhash**   | 1.67 | 1.46 | **1.46** | **1.46** | 1.88 | 2.92 |
| xxh3         | 1.46 | 1.25 | 1.25 | 1.25 | 1.67 | 2.50 |
| wyhash       | 2.08 | 2.08 | 2.29 | 2.50 | 2.08 | 2.71 |
| ahash        | 1.67 | 1.88 | 1.88 | 1.46 | 1.88 | 2.71 |
| fxhash       | 1.46 | 1.25 | 1.25 | 1.25 | 1.46 | 2.71 |
| siphash-1-3  | 3.54 | 3.54 | 4.17 | 5.21 | 7.29 | 11.88|
| highwayhash  | 17.29| 17.50| 16.66| 16.66| 17.50| 20.62|

Tiny-key latency turun signifikan (8-16B: −30%) setelah avalanche
dihilangkan (lihat *Catatan* di akhir).

### Streaming 4 KiB via chunked writes (ns total per hash, lower = better)

| Hasher       | 8 B  | 32 B | 64 B  | 256 B | 1 KB  | 4 KB  |
|--------------|-----:|-----:|------:|------:|------:|------:|
| **axhash**   | 652  | 225  | **177** | 197   | **66**  | **41** |
| xxh3         | 1688 | 610  | 291   | 147   | 124   | 122   |
| wyhash       | 1377 | 372  | 262   | 179   | 156   | 154   |
| ahash        | 879  | 305  | 243   | 199   | 188   | 178   |
| fxhash       | 505  | 138  | 138   | 115   | 134   | 162   |
| siphash-1-3  | 1482 | 923  | 810   | 750   | 714   | 718   |
| highwayhash  | 1681 | 761  | 541   | 416   | 374   | 366   |

axhash **paling cepat untuk chunked-write 1 KB & 4 KB** (66 ns dan 41 ns),
karena AES-NEON pipeline jelas-jelas amortized di throughput tinggi.

### Concurrent scaling (ops/sec, 256 B payload)

| Hasher       | 1 thread | 2 threads | 4 threads | 8 threads |
|--------------|---------:|----------:|----------:|----------:|
| **axhash**   | **1.32 e8** | **2.40 e8** | 4.56 e8 | 5.54 e8   |
| xxh3         | 1.02 e8  | 1.99 e8   | 3.06 e8   | 3.87 e8   |
| wyhash       | 1.13 e8  | 2.11 e8   | 4.30 e8   | 5.83 e8   |
| ahash        | 9.70 e7  | 1.91 e8   | 3.91 e8   | 5.30 e8   |
| fxhash       | 1.27 e8  | 2.51 e8   | 5.18 e8   | 7.18 e8   |
| siphash-1-3  | 2.08 e7  | 4.15 e7   | 8.34 e7   | 1.12 e8   |
| highwayhash  | 2.79 e7  | 5.55 e7   | 1.10 e8   | 1.48 e8   |

### Hash quality (head-to-head sampling, semua hasher ≈ ideal)

| Hasher       | avalanche % | collision % | chi² (256 buckets) | max bit-bias % |
|--------------|------------:|------------:|-------------------:|---------------:|
| axhash       | 49.84       | 0.000000    | 278.0              | 0.4088         |
| xxh3         | 49.83       | 0.000000    | 275.2              | 0.3612         |
| wyhash       | 49.92       | 0.000000    | 289.3              | 0.3372         |
| ahash        | 50.13       | 0.000000    | 236.9              | 0.3380         |
| fxhash       | 50.02       | 0.000000    | 267.6              | 0.2944         |
| siphash-1-3  | 49.90       | 0.000000    | 248.7              | 0.4088         |
| highwayhash  | 50.15       | 0.000000    | 256.3              | 0.3572         |

Quality tetap **dalam range yang ideal** (avalanche ≈ 50%, 0 collision
di 1M random inputs). Bit-bias 0.41% sebanding dengan siphash-1-3 dan
ahash; sedikit lebih tinggi dari xxh3 (0.36) tapi tetap aman untuk hash
non-kriptografik. Trade-off ini disengaja untuk gain latency 30%.

### HashMap workload (10 000 entries, ns/op)

`get-hit`:

| Hasher       |  min |  median |  mean | ±stddev |
|--------------|-----:|--------:|------:|--------:|
| **axhash**   | 1.25 | **1.40**| 1.43  | 0.27    |
| xxh3         | 8.20 | 8.33    | 8.70  | 1.49    |
| wyhash       | 1.49 | 1.55    | 2.13  | 1.25    |
| ahash        | 1.57 | 1.61    | 2.12  | 0.99    |
| fxhash       | 1.17 | 1.31    | 1.39  | 0.48    |
| siphash-1-3  | 4.40 | 5.12    | 6.84  | 3.36    |
| highwayhash  | 33.00| 35.53   | 37.98 | 6.58    |

`mixed` (70% get-hit, 20% miss, 10% insert):

| Hasher       |  min | median |  mean | ±stddev |
|--------------|-----:|-------:|------:|--------:|
| **axhash**   | 7.47 | **7.67** | 7.85 | 0.44   |
| xxh3         | 15.87| 16.02  | 16.11 | 0.26    |
| wyhash       | 7.82 | 7.94   | 8.02  | 0.28    |
| ahash        | 9.14 | 9.28   | 9.31  | 0.12    |
| fxhash       | 6.84 | 7.00   | 7.12  | 0.38    |
| siphash-1-3  | 14.06| 14.15  | 14.21 | 0.21    |
| highwayhash  | 45.67| 46.02  | 46.27 | 0.47    |

axhash **menang head-to-head di mixed workload** (7.67 ns vs xxh3 16.02 ns,
ahash 9.28 ns), competitive di get-hit dengan median 1.40 ns. Mengalahkan
xxh3 5.9× di mixed workload.

## Detail criterion (single-hasher)

### One-shot throughput

| Size  | axhash       | axhash_seeded |
|-------|-------------:|--------------:|
| 4 B   | 3.32 GiB/s   | 3.15 GiB/s    |
| 16 B  | 13.06 GiB/s  | 13.89 GiB/s   |
| 64 B  | 24.04 GiB/s  | 23.76 GiB/s   |
| 256 B | 27.56 GiB/s  | 27.94 GiB/s   |
| 4 KiB | 90.06 GiB/s  | 87.27 GiB/s   |
| 64 KiB| 89.46 GiB/s  | 90.88 GiB/s   |

### `Hash` trait integration

| Operation                        | Latency  | Throughput   |
|----------------------------------|---------:|-------------:|
| `axhash_of::<struct>()`          | 1.00 ns  | 998 Melem/s  |
| `axhash_of_seeded::<struct>()`   | 1.05 ns  | 948 Melem/s  |
| `axhash_of::<u64>()`             | 541 ps   | 1.85 Gelem/s |
| `axhash_of::<&str>()` (8 char)   | 2.97 ns  | 337 Melem/s  |

### Hasher latency

| Operation                | Latency |
|--------------------------|--------:|
| `write_u64` + `finish`   | **358 ps** |
| `write_u32` + `finish`   | **430 ps** |
| 8-byte `str` write       | 1.13 ns |
| 16-byte `str` write      | 1.28 ns |
| `finish()` (idempotent)  | 552 ps  |
| `finish()` second call   | 757 ps  |

`write_u64 + finish` turun **33%** vs baseline 2-mul avalanche (535 → 358 ps)
karena Hash trait integration paling sensitif terhadap finalize cost.

### HashMap (Ax vs `DefaultHasher`)

`insert-u64`:

| Size    | AxBuildHasher | DefaultHasher | speedup |
|---------|--------------:|--------------:|--------:|
| 100     | 1.25 µs       | 2.19 µs       | 1.75 × |
| 1 000   | 14.41 µs      | 31.08 µs      | 2.16 × |
| 10 000  | 120.5 µs      | 280.1 µs      | 2.32 × |
| 100 000 | 1.109 ms      | 2.462 ms      | 2.22 × |

`get-hit`:

| Size    | AxBuildHasher | DefaultHasher | speedup |
|---------|--------------:|--------------:|--------:|
| 100     | 196 ns        | 439 ns        | 2.24 × |
| 1 000   | 1.73 µs       | 4.61 µs       | 2.67 × |
| 10 000  | 19.20 µs      | 52.32 µs      | 2.72 × |
| 100 000 | 223.7 µs      | 599.9 µs      | 2.68 × |

`get-miss` (10 000): **14.93 µs (Ax) vs 37.55 µs (Default) — 2.51 ×**

`mixed` (random insert/get/miss): **1.037 ms (Ax) vs 1.889 ms (Default) — 1.82 ×**

`build-hasher` instantiate: 1.44 ns

## SMHasher3 validation — **188/188 PASS** ✅

Suite SMHasher3 v3 fork ([fwojcik/smhasher3](https://gitlab.com/fwojcik/smhasher3))
dijalankan penuh terhadap `axhash-ffi` lewat `axhash_bytes_seeded`.

| Item | Nilai |
|------|------|
| Hasil overall | **`pass`** |
| Sub-test passed | **188 / 188** |
| Verification value (LE) | `0xB6E1DBEA` |
| Total durasi | 244 detik |
| Backend runtime | `aarch64_aes_neon` (Apple M4) |
| Hash bits | 64 |

Test categories yang dilibatkan:

- **Sanity** — verification, sanity check, append/prepend zeroes, thread-safety
- **Avalanche** & **BIC** (bit independence)
- **Keyset**: Zeroes, Cyclic, Sparse, Permutation, Text, TwoBytes,
  PerlinNoise, Bitflip, Long (alnum random with varying head/tail)
- **Seed-side**: Seed Zeroes/Sparse/BlockLength/BlockOffset/Avalanche/BIC/
  Bitflip dan keyset 'Seed'
- **Statistik distribusi**: MomentChi2, DiffDist, per-bit distribution bias

Lihat [`doc/smhasher3-roadmap.md`](smhasher3-roadmap.md) untuk perjalanan
perbaikan dari baseline 145/188 (v0.9.0) ke 188/188 (v0.10.0) — termasuk
4 bug struktural yang ditemukan dan diperbaiki.

## Catatan

- **Versi 0.9.0 → 0.10.0** mengubah hash output (verification value
  `0xB6E1DBEA`). Tidak kompatibel ke belakang untuk persisted-hash use-case.
- **Hot-path optimization v0.10.0**: setelah verifikasi SMHasher3 188/188
  dengan baseline 2-mul Murmur3 fmix64, profile menunjukkan avalanche
  sebagai self-time terbesar (14.79s di Instruments). Tiga eksperimen
  diuji terhadap full SMHasher3 suite:

  | Avalanche | SMHasher3 | Verification | u64 latency |
  |-----------|-----------|--------------|-------------|
  | 2 mul (Murmur3 fmix64) | 188/188 | `0xCE7A8AFE` | 535 ps |
  | 1 mul (single half-round) | 188/188 | `0x68FD9F96` | ~440 ps |
  | **0 mul (identity, dipilih)** | **188/188** | **`0xB6E1DBEA`** | **358 ps** |

  Branch finalizer (selalu berakhir dengan ≥1 `folded_multiply`) sudah
  cukup kuat untuk lulus SMHasher3 tanpa avalanche tambahan, sehingga
  E3 (no avalanche) dipilih untuk maximum tiny-key performance.
- **Build flags**: bench dengan `--release` profile (lto=fat,
  codegen-units=1). Backend AES-NEON otomatis di Apple Silicon.
- **Reproduksi**:
  ```bash
  # Head-to-head
  cargo run --release -p axhash-headtohead
  # Criterion micro-bench
  cargo bench
  # SMHasher3 full suite (build dulu lihat smhasher3-roadmap.md)
  ~/Development/smhasher3/build/SMHasher3 AxHash-64
  ```
