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

pub(crate) const DEFAULT_ACC: u64 = seed_lane(0, 0);

#[inline(always)]
pub(crate) const fn avalanche(x: u64) -> u64 {
    x
}
