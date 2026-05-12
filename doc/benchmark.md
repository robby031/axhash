# Benchmarks

Benchmarked with Criterion.rs on Apple Silicon (release mode), **AxHash v0.9.0**.

## Performance Highlights

| Benchmark | AxHash v0.9.0 | v0.8.2 |
|-----------|-------------:|-------:|
| HashMap `get-hit` (100k keys) | ~469 Melem/s | ~494 |
| HashMap `get-miss` (10k keys) | ~738 Melem/s | ~785 |
| HashMap `insert-u64` (100k keys) | ~90 Melem/s | ~90 |
| `u64` hashing throughput | ~1.99 Gelem/s | ~2.0 |
| Thread-local concurrent hashing (8 threads) | ~750 Melem/s | ~752 |
| Large-buffer throughput (64K) | ~95.6 GiB/s | ~94 |
| Large-buffer throughput (4K) | ~99.2 GiB/s | ~100 |
| `write_u64` latency | ~374 ps | ~3 ns |
| `write_u128` latency | ~370 ps | ~1.48 ns |
| Mixed struct hashing | ~289 Melem/s | ~290 |
| `HashMap` builder creation | ~1.29 ns | ~1.3 ns |

## Real-world HashMap Comparison

| Workload | AxHash v0.9.0 | DefaultHasher |
|----------|-------------:|--------------:|
| `insert-u64` (100k) | ~90 Melem/s | ~45 Melem/s |
| `get-hit` (100k) | ~469 Melem/s | ~178 Melem/s |
| `get-miss` (10k) | ~738 Melem/s | ~280 Melem/s |
| Mixed workload | ~13.0 Melem/s | ~6.4 Melem/s |

## Concurrent Scaling

| Threads | Throughput | vs v0.8.2 |
|---------|-----------:|----------:|
| 1 | ~231 Melem/s | ~230 |
| 2 | ~421 Melem/s | ~420 |
| 4 | ~699 Melem/s | ~691 |
| 8 | ~750 Melem/s | ~752 |

## One-shot Throughput by Input Size

| Size | axhash | axhash_seeded |
|------|--------|--------------:|
| 4 B | ~3.71 GiB/s | ~3.71 GiB/s |
| 16 B | ~18.6 GiB/s | ~18.7 GiB/s |
| 64 B | ~24.2 GiB/s | ~24.4 GiB/s |
| 256 B | ~59.4 GiB/s | ~59.8 GiB/s |
| 4 KiB | ~97.6 GiB/s | ~98.5 GiB/s |
| 64 KiB | ~92.9 GiB/s | ~95.6 GiB/s |

## Notes

- **Small-key latency** (`axhash/4B`, `axhash/16B`) shows higher ns in v0.9.0 because the baseline comparison in Criterion was against an older, faster reference; the absolute values remain sub-nanosecond.
- **Wrapper API** (`hash()`, `hash_with_seed()`) measured ~10× faster throughput in v0.9.0 vs previous baseline due to fixed inlining/devirtualization in the new module layout.
- **Seeded 256B** throughput improved **~135%** (58 GiB/s vs ~25 GiB/s previously) thanks to the consolidated scalar path in `backend/scalar.rs`.
- See `HASILUJI.md` for raw Criterion output and statistical change analysis.
