# axhash-ffi

`axhash-ffi` exposes AxHash through a stable C ABI.

This package is the recommended entrypoint for C, C++, Go, Zig, Swift, Kotlin Native, and any environment that can call C-compatible symbols.

## Build

```bash
cargo build -p axhash-ffi --release
```

Generated header:

```text
crates/axhash-ffi/include/axhash.h
```

Typical native outputs:

- macOS: `libaxhash_ffi.a`, `libaxhash_ffi.dylib`
- Linux: `libaxhash_ffi.a`, `libaxhash_ffi.so`
- Windows: `axhash_ffi.lib`, `axhash_ffi.dll`

## One-shot Example

```c
#include "axhash.h"
#include <stdint.h>
#include <stdio.h>

int main(void) {
    const uint8_t data[] = "hello axhash";
    uint64_t digest = axhash_bytes(data, sizeof(data) - 1);
    uint64_t seeded = axhash_bytes_seeded(data, sizeof(data) - 1, 0x12345678ULL);

    printf("%016llx\n", (unsigned long long)digest);
    printf("%016llx\n", (unsigned long long)seeded);
    return 0;
}
```

## Streaming Example

```c
#include "axhash.h"
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

int main(void) {
    AxHashState *state = axhash_hasher_new_seeded(0x4444ULL);
    if (!state) {
        return 1;
    }

    bool ok1 = axhash_hasher_write(state, (const uint8_t *)"hello ", 6);
    bool ok2 = axhash_hasher_write(state, (const uint8_t *)"world", 5);
    uint64_t digest = axhash_hasher_finish(state);
    axhash_hasher_free(state);

    if (!ok1 || !ok2) {
        return 1;
    }

    printf("%016llx\n", (unsigned long long)digest);
    return 0;
}
```

## Batch Example

```c
#include "axhash.h"
#include <stdint.h>
#include <stdio.h>

int main(void) {
    const uint8_t a[] = "alpha";
    const uint8_t b[] = "beta";
    const AxHashIovec jobs[2] = {
        { a, sizeof(a) - 1 },
        { b, sizeof(b) - 1 },
    };
    uint64_t out[2] = {0, 0};

    axhash_batch_seeded(jobs, 2, 0x1234ULL, out);

    printf("%016llx\n", (unsigned long long)out[0]);
    printf("%016llx\n", (unsigned long long)out[1]);
    return 0;
}
```

## Runtime Information

```c
#include "axhash.h"
#include <stdio.h>

int main(void) {
    printf("%d\n", (int)axhash_runtime_backend());
    printf("%d\n", (int)axhash_runtime_has_aes());
    return 0;
}
```

## License

MIT.
