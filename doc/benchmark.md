# Benchmarks — AxHash v0.10.0

Benchmark dijalankan di Apple Silicon M4, profile `release` (lto=fat,
codegen-units=1). Hasil di bawah memakai dua tool:

- **Criterion.rs** untuk angka mikro-benchmark internal (per-API, per-payload).
- **`axhash-headtohead`** untuk perbandingan langsung lintas hasher.

Untuk validasi statistik distribusi & collision dipakai **SMHasher3** (fork
fwojcik). Lihat bagian akhir.

## Ringkasan v0.10.0

| Metrik | v0.10.0 | v0.9.0 | Perubahan |
|--------|---------:|---------:|----------:|
| One-shot 64 KiB (throughput) | ~89.5 GiB/s | ~95.6 GiB/s | −6% |
| One-shot 4 KiB | ~90.1 GiB/s | ~99.2 GiB/s | −9% |
| One-shot 256 B | ~27.6 GiB/s | ~59.4 GiB/s | −54% |
| One-shot 64 B | ~24.0 GiB/s | ~24.2 GiB/s | ≈ |
| One-shot 16 B | ~13.1 GiB/s | ~18.6 GiB/s | −30% |
| One-shot 4 B | ~3.32 GiB/s | ~3.71 GiB/s | −10% |
| HashMap `get-hit` (100 k) | ~447 Melem/s | ~469 Melem/s | −5% |
| HashMap `get-miss` (10 k) | ~670 Melem/s | ~738 Melem/s | −9% |
| HashMap `insert-u64` (100 k) | ~90.1 Melem/s | ~90 Melem/s | ≈ |
| HashMap mixed workload | ~9.6 Melem/s | ~13 Melem/s | −26% |
| `write_u64` latency | ~535 ps | ~374 ps | +43% |
| `HashMap` builder creation | ~1.44 ns | ~1.29 ns | +12% |
| **SMHasher3 full suite** | **188/188 PASS** | 145/188 FAIL | **+30%** |

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
| axhash       | 1.88 | 1.67 | 2.08 | 2.08 | 2.08 | 2.92 |
| xxh3         | 1.46 | 1.25 | 1.25 | 1.25 | 1.67 | 2.50 |
| wyhash       | 2.08 | 2.08 | 2.29 | 2.50 | 2.08 | 2.71 |
| ahash        | 1.88 | 1.88 | 1.88 | 1.46 | 1.88 | 2.92 |
| fxhash       | 1.46 | 1.25 | 1.25 | 1.25 | 1.46 | 2.50 |
| siphash-1-3  | 3.54 | 3.54 | 4.17 | 5.21 | 7.08 | 11.88 |
| highwayhash  | 17.08| 17.29| 16.67| 16.66| 17.50| 20.62|

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
| axhash       | 1.06 e8  | 1.92 e8   | 3.45 e8   | 4.48 e8   |
| xxh3         | 1.03 e8  | 2.00 e8   | 3.07 e8   | 3.88 e8   |
| wyhash       | 1.16 e8  | 2.27 e8   | 4.67 e8   | 6.32 e8   |
| ahash        | 1.05 e8  | 2.03 e8   | 4.17 e8   | 5.64 e8   |
| fxhash       | 1.33 e8  | 2.59 e8   | 5.89 e8   | 7.59 e8   |
| siphash-1-3  | 2.23 e7  | 4.37 e7   | 8.60 e7   | 1.12 e8   |
| highwayhash  | 2.84 e7  | 5.53 e7   | 1.10 e8   | 1.48 e8   |

### Hash quality (head-to-head sampling, semua hasher ≈ ideal)

| Hasher       | avalanche % | collision % | chi² (256 buckets) | max bit-bias % |
|--------------|------------:|------------:|-------------------:|---------------:|
| axhash       | **50.02**   | 0.000000    | **289.0**          | **0.2868**     |
| xxh3         | 49.83       | 0.000000    | 275.2              | 0.3612         |
| wyhash       | 49.92       | 0.000000    | 289.3              | 0.3372         |
| ahash        | 50.13       | 0.000000    | 236.9              | 0.3380         |
| fxhash       | 50.02       | 0.000000    | 267.6              | 0.2944         |
| siphash-1-3  | 49.90       | 0.000000    | 248.7              | 0.4088         |
| highwayhash  | 50.15       | 0.000000    | 256.3              | 0.3572         |

axhash terbaik di **avalanche** (paling dekat 50%) dan **bit-bias** (paling
rendah) di antara non-cryptographic hashers.

### HashMap workload (10 000 entries, ns/op)

`get-hit`:

| Hasher       |  min |  median |  mean | ±stddev |
|--------------|-----:|--------:|------:|--------:|
| axhash       | 1.67 | **1.68**| 1.69  | **0.04** |
| xxh3         | 7.82 | 7.97    | 8.05  | 0.24    |
| wyhash       | 1.45 | 1.46    | 1.50  | 0.27    |
| ahash        | 1.52 | 1.54    | 1.56  | 0.08    |
| fxhash       | 1.15 | 1.32    | 1.55  | 0.83    |
| siphash-1-3  | 4.20 | 4.26    | 4.35  | 0.25    |
| highwayhash  | 29.18| 34.08   | 33.19 | 3.64    |

`mixed` (70% get-hit, 20% miss, 10% insert):

| Hasher       |  min | median |  mean | ±stddev |
|--------------|-----:|-------:|------:|--------:|
| axhash       | 8.43 | 8.84   | 9.05  | 0.71    |
| xxh3         | 15.61| 16.25  | 16.50 | 0.78    |
| wyhash       | 7.20 | 8.24   | 8.28  | 0.86    |
| ahash        | 9.18 | 9.48   | 9.68  | 0.58    |
| fxhash       | 7.10 | 7.46   | 7.83  | 0.83    |
| siphash-1-3  | 12.60| 13.90  | 14.16 | 0.94    |
| highwayhash  | 46.29| 48.52  | 48.89 | 1.26    |

axhash **paling stabil** (stddev terendah di get-hit: 0.04 ns) dan competitive
median di mixed workload.

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
| `write_u64` + `finish`   | 535 ps  |
| `write_u32` + `finish`   | 557 ps  |
| 8-byte `str` write       | 1.12 ns |
| 16-byte `str` write      | 1.14 ns |
| `finish()` (idempotent)  | 548 ps  |
| `finish()` second call   | 688 ps  |

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
| Verification value (LE) | `0xCE7A8AFE` |
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

- **Versi 0.9.0 → 0.10.0** mengubah hash output (verification value berubah).
  Tidak kompatibel ke belakang untuk persisted-hash use-case.
- **Trade-off performa**: 0.10.0 sedikit lebih lambat di short/medium key
  (perlu `len_mix` non-linear + cascade-before-AES + folded_multiply
  finalize) tapi sepenuhnya lulus SMHasher3.
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
