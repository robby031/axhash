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

    #[inline(always)]
    pub(crate) fn flush_sponge_hot(&mut self) {
        let lo = self.sponge as u64;
        let hi = (self.sponge >> 64) as u64;
        self.acc = folded_multiply(lo ^ self.acc, hi ^ SECRET[1]);
        self.sponge = 0;
        self.sponge_bits = 0;
    }

    #[inline(never)]
    #[cold]
    pub(crate) fn flush_sponge_slow(&mut self) {
        self.flush_sponge_hot();
    }

    #[inline(always)]
    pub(crate) fn push_num<T: Into<u128>>(&mut self, value: T, bits: u8) {
        if self.sponge_bits as u16 + bits as u16 > 128 {
            self.flush_sponge();
        }

        self.sponge |= value.into() << self.sponge_bits;
        self.sponge_bits += bits;
    }

    #[inline(always)]
    pub(crate) fn push_bytes(&mut self, bytes: &[u8]) {
        let n = bytes.len();
        debug_assert!(n + ((self.sponge_bits >> 3) as usize) <= 16);
        if n == 0 {
            return;
        }

        // Common sizes: single load tanpa buffer init.
        let value: u128 = if n == 8 {
            let arr: [u8; 8] = bytes.try_into().unwrap();
            u64::from_le_bytes(arr) as u128
        } else if n == 4 {
            let arr: [u8; 4] = bytes.try_into().unwrap();
            u32::from_le_bytes(arr) as u128
        } else if n == 16 {
            let arr: [u8; 16] = bytes.try_into().unwrap();
            u128::from_le_bytes(arr)
        } else if n == 1 {
            bytes[0] as u128
        } else if n == 2 {
            let arr: [u8; 2] = bytes.try_into().unwrap();
            u16::from_le_bytes(arr) as u128
        } else {
            // 3, 5, 6, 7, 9..=15: zero-extend ke 16-byte buf
            let mut buf = [0u8; 16];
            buf[..n].copy_from_slice(bytes);
            u128::from_le_bytes(buf)
        };

        self.sponge |= value << self.sponge_bits;
        self.sponge_bits += (n << 3) as u8;
    }
}

impl Default for AxHasher {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}
