use crate::constants::SECRET;
use crate::math::folded_multiply;

#[derive(Clone, Debug)]
pub struct AxHasher {
    pub(crate) acc: u64,
    pub(crate) sponge: u128,
    pub(crate) sponge_bits: u8,
}

impl AxHasher {
    #[inline(always)]
    pub fn new() -> Self {
        Self::new_with_seed(0)
    }

    #[inline(always)]
    pub fn new_with_seed(seed: u64) -> Self {
        Self {
            acc: seed ^ SECRET[0],
            sponge: 0,
            sponge_bits: 0,
        }
    }

    #[inline(always)]
    pub(crate) fn flush_sponge(&mut self) {
        if self.sponge_bits != 0 {
            self.flush_sponge_slow();
        }
    }

    #[inline(never)]
    #[cold]
    pub(crate) fn flush_sponge_slow(&mut self) {
        let lo = self.sponge as u64;
        let hi = (self.sponge >> 64) as u64;
        self.acc = folded_multiply(lo ^ self.acc, hi ^ SECRET[1]);
        self.sponge = 0;
        self.sponge_bits = 0;
    }

    #[inline(always)]
    pub(crate) fn push_num<T: Into<u128>>(&mut self, value: T, bits: u8) {
        if self.sponge_bits as u16 + bits as u16 > 128 {
            self.flush_sponge();
        }

        self.sponge |= value.into() << self.sponge_bits;
        self.sponge_bits += bits;
    }

}

impl Default for AxHasher {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}
