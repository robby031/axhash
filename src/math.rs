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

#[inline(always)]
pub(crate) fn avalanche(mut x: u64) -> u64 {
    x ^= x >> 32;
    x = folded_multiply(x ^ SECRET[1], x.rotate_left(19) ^ STRIPE_SECRET[2]);
    x ^= x >> 29;
    x = folded_multiply(x ^ SECRET[3], x.rotate_left(27) ^ STRIPE_SECRET[0]);
    x ^ (x >> 28)
}
