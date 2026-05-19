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

## Ecosystem

| Crate | Description |
|-------|-------------|
| `axhash` | High-performance hashing engine |
| `axhash-map` | Fast HashMap/HashSet powered by `hashbrown` |
| `axhash-indexmap` | Ordered maps with AxHash |

## Documentation

- [Usage Guide](doc/usage.md) — Quick start, API examples, package selection
- [Architecture](doc/architecture.md) — Directory layout, design principles
- [Algorithm](doc/algorithm.md) — Hash algorithm overview, dispatch paths, limitations
- [Benchmarks](doc/benchmark.md) — Performance numbers and scaling data

## Workspace Layout

- [axhash-core](crates/axhash-core/README.md): low-level Rust core and `no_std` engine
- [axhash-ffi](crates/axhash-ffi/README.md): stable C ABI

## License

MIT
