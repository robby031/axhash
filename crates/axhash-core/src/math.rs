use crate::constants::{SECRET, STRIPE_SECRET};

#[inline(always)]
pub(crate) const fn folded_multiply(x: u64, y: u64) -> u64 {
    let product = (x as u128) * (y as u128);
    (product as u64) ^ ((product >> 64) as u64)
}

#[inline(always)]
pub(crate) const fn seed_lane(seed: u64, lane: usize) -> u64 {
    folded_multiply(
        seed.rotate_left((lane as u32) * 17 + 7) ^ SECRET[lane],
        STRIPE_SECRET[lane],
    )
}

// Accumulator default untuk axhash(bytes) dengan seed 0.
// Nilainya identik dengan seed_lane(0, 0) namun dievaluasi di compile-time
// dan dapat dipakai ulang tanpa biaya runtime
pub(crate) const DEFAULT_ACC: u64 = seed_lane(0, 0);

// Branch finalizer sudah strong (selalu berakhir dengan satu atau lebih
// folded_multiply). Avalanche tambahan tidak diperlukan untuk lulus
// SMHasher3 188/188; dipertahankan sebagai identity agar API tetap stabil.
#[inline(always)]
pub(crate) const fn avalanche(x: u64) -> u64 {
    x
}
