# Architecture

Current codebase structure after restructuring.

## Directory Layout

```
src/
├── lib.rs              # Re-exports + runtime API (~37 lines)
├── constants.rs        # Hash constants (SECRET, STRIPE_SECRET, FINAL_MIX)
├── math.rs             # folded_multiply, avalanche, seed_lane
├── memory.rs           # Unsafe unaligned memory reads (r_u64, r_128)
├── backend/            # Hashing backends
│   ├── mod.rs          # Dispatch (hash_bytes_core, selected_backend)
│   ├── scalar.rs       # All scalar hash paths + helper
│   ├── x86_64.rs       # AES-NI / AVX2 long-input path
│   └── aarch64.rs      # AES / NEON long-input path
├── hasher/             # Hasher trait + public API
│   ├── mod.rs          # Re-exports
│   ├── core.rs         # AxHasher struct + state
│   ├── build.rs        # AxBuildHasher + seed generation
│   ├── api.rs          # One-shot functions (axhash, axhash_seeded, ...)
│   └── trait_impl.rs   # Hasher trait implementation
└── tests/              # Test modules (moved from lib.rs)
    ├── mod.rs          # Shared helpers (DemoRecord, chi_squared, ...)
    ├── determinism.rs
    ├── buildhasher.rs
    ├── trait_contract.rs
    ├── backend_parity.rs
    ├── lower_bits.rs
    ├── collisions.rs
    └── predictability.rs
```

## Design Principles

1. **Backend dispatch** — `backend/mod.rs` selects scalar, x86_64 AES/AVX2, or aarch64 AES/NEON at runtime (or compile-time for `no_std`).
2. **Short inputs are identical across all backends** — only long inputs (>128 bytes) use platform-specific SIMD.
3. **No hidden state** — `AxHasher` is pure: same seed + same input always yields same output.
4. **Tests grouped by concern** — determinism, buildhasher, trait contract, backend parity, lower-bit distribution, collisions, predictability.
