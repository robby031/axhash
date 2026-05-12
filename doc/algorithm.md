# Algorithm

AxHash is a 64-bit non-cryptographic hash function optimized for HashMap workloads.

## Overview

- **Output:** 64-bit digest
- **State:** 64-bit accumulator (`acc`) + 64-bit sponge (for streaming)
- **Seed handling:** Raw seed is mixed with secret constants via `seed_lane()` before use
- **Finalization:** `avalanche()` applies bit-mixing to ensure uniform lower-bit distribution

## Input-Length Dispatch

`hash_bytes_core` selects a path based on input length:

| Length | Path | Backend |
|--------|------|---------|
| 0 | Return `acc` | — |
| 1–16 | `hash_bytes_short` | Scalar (all platforms) |
| 17–32 | `hash_bytes_17_32` | Scalar (all platforms) |
| 33–64 | `hash_bytes_33_64` | Scalar (all platforms) |
| 65–128 | `hash_bytes_65_128` | Scalar (all platforms) |
| >128 | `hash_bytes_long` | **Platform-specific** |

## Short Paths (≤128 bytes)

All platforms share the same scalar implementation for short inputs. Each path loads fixed-width 64-bit chunks from the start and end of the buffer, mixes them with secret constants via `folded_multiply`, and XOR-rotates the intermediate results.

## Long Path (>128 bytes)

The long path is platform-specific for performance:

- **Scalar** (`backend/scalar.rs`): 128-byte block processing with folded-multiply mixing. A `mix_128_block` helper processes each block to avoid loop/tail duplication.
- **x86_64** (`backend/x86_64.rs`): AES-NI + AVX2 — loads 256-bit pairs with `_mm256_loadu_si256`, applies `_mm_aesenc_si128` rounds, and XOR-reduces vectors. Uses `process_128_block_x86` helper.
- **aarch64** (`backend/aarch64.rs`): AES + NEON — loads 128-bit chunks with `r_128`, applies `vaeseq_u8` + `vaesmcq_u8` rounds, and XOR-reduces vectors. Uses `process_128_block_aarch64` helper.

## Known Limitations

- **Cross-platform long-input divergence:** SIMD backends use different algorithms than scalar for inputs >128 bytes. Short inputs (≤128) are identical everywhere.
- **Seed derivation inconsistency:** `AxHasher::new_with_seed(seed)` and `axhash_seeded(bytes, seed)` use different seed preparation. This is documented but not yet unified (breaking change).
