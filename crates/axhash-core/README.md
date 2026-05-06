# axhash-core

`axhash-core` is the low-level Rust engine behind the AxHash family.

Most Rust users should prefer the top-level `axhash` crate because it provides a simpler import path. Use `axhash-core` when you want direct access to the core engine, need `no_std`, or are building your own bindings.

## Installation

```toml
[dependencies]
axhash-core = "0.8"
```

## Features

- `no_std` compatible
- zero allocation on the main hashing path
- deterministic hashing with optional seeds
- streaming hasher implementation
- `HashMap` integration through `AxBuildHasher`
- runtime backend detection

## Examples

Hash bytes with the default seed:

```rust
use axhash_core::axhash;

fn main() {
    let digest = axhash(b"hello axhash");
    println!("{digest:016x}");
}
```

Hash bytes with a custom seed:

```rust
use axhash_core::axhash_seeded;

fn main() {
    let digest = axhash_seeded(b"hello axhash", 0x1234_5678);
    println!("{digest:016x}");
}
```

Hash any value that implements `Hash`:

```rust
use axhash_core::axhash_of_seeded;

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

    let digest = axhash_of_seeded(&key, 0xdead_beef);
    println!("{digest:016x}");
}
```

Use the streaming hasher:

```rust
use axhash_core::AxHasher;
use std::hash::Hasher as _;

fn main() {
    let mut hasher = AxHasher::new_with_seed(0x1234);
    hasher.write(b"part-1");
    hasher.write(b"part-2");

    println!("{:016x}", hasher.finish());
}
```

Use AxHash in `HashMap`:

```rust
use axhash_core::AxBuildHasher;
use std::collections::HashMap;

fn main() {
    let mut map: HashMap<&str, i32, AxBuildHasher> =
        HashMap::with_hasher(AxBuildHasher::with_seed(0xfeed_beef));

    map.insert("alpha", 1);
    map.insert("beta", 2);

    println!("{:?}", map.get("alpha"));
}
```

Check the active backend:

```rust
use axhash_core::{runtime_backend, runtime_has_aes};

fn main() {
    println!("{:?}", runtime_backend());
    println!("{}", runtime_has_aes());
}
```

## License

MIT.
