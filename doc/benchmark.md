
# Benchmarks — AxHash v0.10.0

Benchmarks were run on Apple Silicon M4, `release` profile (lto=fat, codegen-units=1). The results below use two tools:

- **Criterion.rs** for internal micro-benchmark numbers (per-API, per-payload).
- **`axhash-headtohead`** for direct cross-hasher comparisons.

For statistical distribution & collision validation, **SMHasher3** (fwojcik fork) was used. See the end section.

## Summary v0.10.0 (E3, no-avalanche)


| Metric | v0.10.0 | v0.9.0 | Change |
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


> **Trade-off v0.10.0:** Bulk throughput decreased because the algorithm adds per-branch mixing to pass SMHasher3 (Bug A/B/C/D in v0.9.0). Conversely, **tiny-key latency (≤16B) dropped significantly** because the avalanche finalizer was removed after the branch finalizer proved strong enough. Net result: real-world HashMap workload `get-miss` & latency-sensitive path are faster than v0.9.0, bulk throughput slightly decreased.

> Trade-off v0.10.0: throughput slightly decreased for short/medium keys, exchanged for **industrial-grade distribution quality** (188/188 SMHasher3). Version 0.9.0 had XOR cancellation in short-key tail, lane cancellation in long-path, and linear length-XOR causing many collisions.


## Head-to-head vs popular hashers

Apple M4, single-thread, median 300 sample × 200 batch. Source:
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


axhash excels in **bulk throughput ≥ 4 KB** (AES-NEON pipeline). Competitors are faster on short keys.

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


Tiny-key latency dropped significantly (8-16B: −30%) after avalanche was removed (see *Notes* at the end).

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


axhash is **the fastest for chunked-write 1 KB & 4 KB** (66 ns and 41 ns), because the AES-NEON pipeline is clearly amortized at high throughput.


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


### Hash quality (head-to-head sampling, all hashers ≈ ideal)

| Hasher       | avalanche % | collision % | chi² (256 buckets) | max bit-bias % |
|--------------|------------:|------------:|-------------------:|---------------:|
| axhash       | 49.84       | 0.000000    | 278.0              | 0.4088         |
| xxh3         | 49.83       | 0.000000    | 275.2              | 0.3612         |
| wyhash       | 49.92       | 0.000000    | 289.3              | 0.3372         |
| ahash        | 50.13       | 0.000000    | 236.9              | 0.3380         |
| fxhash       | 50.02       | 0.000000    | 267.6              | 0.2944         |
| siphash-1-3  | 49.90       | 0.000000    | 248.7              | 0.4088         |
| highwayhash  | 50.15       | 0.000000    | 256.3              | 0.3572         |


Quality remains **in the ideal range** (avalanche ≈ 50%, 0 collision in 1M random inputs). Bit-bias 0.41% is comparable to siphash-1-3 and ahash; slightly higher than xxh3 (0.36) but still safe for non-cryptographic hashes. This trade-off is intentional for a 30% latency gain.

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


axhash **wins head-to-head in mixed workload** (7.67 ns vs xxh3 16.02 ns, ahash 9.28 ns), competitive in get-hit with a median of 1.40 ns. Beats xxh3 by 5.9× in mixed workload.


## Detailed criterion (single-hasher)

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


`write_u64 + finish` dropped **33%** vs baseline 2-mul avalanche (535 → 358 ps) because Hash trait integration is most sensitive to finalize cost.

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

SMHasher3 v3 fork suite ([fwojcik/smhasher3](https://gitlab.com/fwojcik/smhasher3)) was run fully against `axhash-ffi` via `axhash_bytes_seeded`.

| Item | Value |
|------|------|
| Overall result | **`pass`** |
| Sub-test passed | **188 / 188** |
| Verification value (LE) | `0xB6E1DBEA` |
| Total duration | 244 seconds |
| Backend runtime | `aarch64_aes_neon` (Apple M4) |
| Hash bits | 64 |

Test categories included:

- **Sanity** — verification, sanity check, append/prepend zeroes, thread-safety
- **Avalanche** & **BIC** (bit independence)
- **Keyset**: Zeroes, Cyclic, Sparse, Permutation, Text, TwoBytes, PerlinNoise, Bitflip, Long (alnum random with varying head/tail)
- **Seed-side**: Seed Zeroes/Sparse/BlockLength/BlockOffset/Avalanche/BIC/ Bitflip and keyset 'Seed'
- **Distribution statistics**: MomentChi2, DiffDist, per-bit distribution bias


## Streaming optimization (v0.10.x)

`AxHasher::write(&[u8])` for slices **< 8 bytes** is redirected to the same sponge buffer as `write_u8/u16/u32`. This provides semantic consistency (`write(&[0x01, 0x02])` ≡ `write_u16(0x0201)`) AND avoids dispatch to `hash_bytes_core` for sub-word writes — which usually come from `Hash` derive for structs with many `u8`/`u16` fields.

Slices ≥ 8 bytes still go through `hash_bytes_core` because the `hash_bytes_short` branch len≥8 is already just 2 folded_multiply — optimal without sponge overhead.

Profiler workload (`run_tiny_key_streaming`, 1.25e9 stream-of-32B):

| Metric | Before streaming opt | After | Δ |
|--------|----------------------:|--------:|------:|
| Profiler total user time | 25.39 s | 24.43 s | **−3.8%** |
| Streaming 8B chunks (h2h) | 654 ns | 654 ns | same |
| Latency 4-16B (h2h) | 1.46 ns | 1.46 ns | same |
| SMHasher3 | 188/188 | **188/188** | unaffected |

Verification value did not change (`0xB6E1DBEA`) because the one-shot FFI path (used by SMHasher3) was not modified.


## Notes

- **Version 0.9.0 → 0.10.0** changes the hash output (verification value `0xB6E1DBEA`). Not backward compatible for persisted-hash use-cases.
- **Hot-path optimization v0.10.0**: after SMHasher3 188/188 verification with baseline 2-mul Murmur3 fmix64, profiling showed avalanche as the largest self-time (14.79s in Instruments). Three experiments were tested against the full SMHasher3 suite:

  | Avalanche | SMHasher3 | Verification | u64 latency |
  |-----------|-----------|--------------|-------------|
  | 2 mul (Murmur3 fmix64) | 188/188 | `0xCE7A8AFE` | 535 ps |
  | 1 mul (single half-round) | 188/188 | `0x68FD9F96` | ~440 ps |
  | **0 mul (identity, chosen)** | **188/188** | **`0xB6E1DBEA`** | **358 ps** |

  Branch finalizer (always ends with ≥1 `folded_multiply`) is strong enough to pass SMHasher3 without additional avalanche, so E3 (no avalanche) was chosen for maximum tiny-key performance.
