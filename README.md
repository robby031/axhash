# AxHash

[![axhash](https://img.shields.io/crates/v/axhash.svg)](https://crates.io/crates/axhash)
[![core](https://img.shields.io/crates/v/axhash-core.svg?label=axhash-core)](https://crates.io/crates/axhash-core)
[![docs-axhash](https://docs.rs/axhash/badge.svg?label=docs-axhash)](https://docs.rs/axhash)
[![docs-core](https://docs.rs/axhash-core/badge.svg?label=docs-core)](https://docs.rs/axhash-core)

[![downloads-axhash](https://img.shields.io/crates/d/axhash.svg?label=axhash%20downloads)](https://crates.io/crates/axhash)
[![downloads-core](https://img.shields.io/crates/d/axhash-core.svg?label=core%20downloads)](https://crates.io/crates/axhash-core)
[![license](https://img.shields.io/crates/l/axhash.svg)](LICENSE)
[![CI](https://github.com/robby031/axhash/actions/workflows/ci.yml/badge.svg)](https://github.com/robby031/axhash/actions)

[![rust](https://img.shields.io/badge/rust-1.85%2B-orange)](https://www.rust-lang.org)
[![no_std](https://img.shields.io/badge/no__std-supported-success)](#)

AxHash is a high-performance deterministic hashing family for Rust and native systems languages.

If you only need AxHash in Rust, start with the `axhash` crate from this workspace. It is the simplest entrypoint and re-exports the core engine with a friendlier import path.

AxHash is optimized for real-world `HashMap` workloads, concurrent systems, cache-heavy applications, and high-throughput native runtimes.

## Features

- Deterministic hashing
- Seeded hashing support
- Streaming API
- Fast `HashMap` integration
- `no_std` support
- Cross-language FFI
- Concurrent-friendly design

## Performance Highlights

Benchmarked with Criterion.rs on Apple Silicon (release mode).

| Benchmark                                   | AxHash Performance |
| ------------------------------------------- | -----------------: |
| HashMap `get-hit` (100k keys)               |       ~494 Melem/s |
| HashMap `get-miss` (10k keys)               |       ~785 Melem/s |
| HashMap `insert-u64` (100k keys)            |        ~90 Melem/s |
| `u64` hashing throughput                    |       ~2.0 Gelem/s |
| Thread-local concurrent hashing (8 threads) |       ~750 Melem/s |
| Large-buffer throughput (64K)               |          ~94 GiB/s |
| Large-buffer throughput (4K)                |         ~100 GiB/s |
| `write_u64` latency                         |              ~3 ns |
| `write_u128` latency                        |           ~1.48 ns |
| Mixed struct hashing                        |       ~290 Melem/s |
| `HashMap` builder creation                  |            ~1.3 ns |

## Real-world HashMap Comparison

| Workload            |        AxHash | DefaultHasher |
| ------------------- | ------------: | ------------: |
| `insert-u64` (100k) |   ~90 Melem/s |   ~44 Melem/s |
| `get-hit` (100k)    |  ~494 Melem/s |  ~183 Melem/s |
| `get-miss` (10k)    |  ~785 Melem/s |  ~280 Melem/s |
| Mixed workload      | ~11.9 Melem/s |  ~6.1 Melem/s |

## Concurrent Scaling

| Threads |   Throughput |
| ------- | -----------: |
| 1       | ~230 Melem/s |
| 2       | ~420 Melem/s |
| 4       | ~691 Melem/s |
| 8       | ~752 Melem/s |

See full Criterion benchmark reports for detailed graphs and methodology.

## Pick The Right Package

- Rust: `axhash`
- C / C++ / Go / Zig / Swift / Kotlin Native: `axhash-ffi`
- Low-level engine / `no_std`: `axhash-core`

## Rust Quick Start

Add the simplest Rust package:

```toml
[dependencies]
axhash = "0.8"
```

Hash raw bytes:

```rust
use axhash::hash;

fn main() {
    let digest = hash(b"hello axhash");
    println!("{digest:016x}");
}
```

Hash raw bytes with a seed:

```rust
use axhash::hash_with_seed;

fn main() {
    let digest = hash_with_seed(b"hello axhash", 0x1234_5678);
    println!("{digest:016x}");
}
```

Hash any Rust value that implements `Hash`:

```rust
use axhash::hash_value;

#[derive(Hash)]
struct SessionKey {
    account_id: u64,
    region_id: u32,
    flags: u32,
}

fn main() {
    let key = SessionKey {
        account_id: 42,
        region_id: 7,
        flags: 3,
    };

    let digest = hash_value(&key);
    println!("{digest:016x}");
}
```

Use the streaming hasher:

```rust
use axhash::AxHasher;
use std::hash::Hasher as _;

fn main() {
    let mut hasher = AxHasher::new_with_seed(0x4444);

    hasher.write(b"hello ");
    hasher.write(b"world");

    println!("{:016x}", hasher.finish());
}
```

Use AxHash with `HashMap`:

```rust
use axhash::AxBuildHasher;
use std::collections::HashMap;

fn main() {
    let mut map =
        HashMap::with_hasher(AxBuildHasher::with_seed(0xfeed_beef));

    map.insert("status", "ok");
    map.insert("runtime", "fast");

    println!("{:?}", map.get("status"));
}
```

Inspect the active backend:

```rust
use axhash::{runtime_backend, runtime_has_aes};

fn main() {
    println!("{:?}", runtime_backend());
    println!("{}", runtime_has_aes());
}
```

## Workspace Layout

- [axhash-core](crates/axhash-core/README.md): low-level Rust core and `no_std` engine
- [axhash-ffi](crates/axhash-ffi/README.md): stable C ABI

## License

MIT
