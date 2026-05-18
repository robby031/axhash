use crate::backend::finalize_vector;
use crate::constants::{SECRET, STRIPE_SECRET};

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "aes")]
unsafe fn aes_round(state: __m128i, block: __m128i) -> __m128i {
    _mm_aesenc_si128(state, block)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn load_pair(ptr: *const u8) -> (__m128i, __m128i) {
    // SAFETY: _mm256_loadu_si256 ("u" = unaligned) is safe for any alignment.
    let pair = unsafe { _mm256_loadu_si256(ptr.cast::<__m256i>()) };
    (
        _mm256_castsi256_si128(pair),
        _mm256_extracti128_si256(pair, 1),
    )
}

#[inline(always)]
unsafe fn lanes(vec: __m128i) -> [u64; 2] {
    // SAFETY: __m128i and [u64; 2] are both 128 bits with identical layout.
    unsafe { core::mem::transmute(vec) }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "aes", enable = "avx2")]
#[inline]
unsafe fn process_128_block_x86(
    ptr: *const u8,
    v0: &mut __m128i,
    v1: &mut __m128i,
    v2: &mut __m128i,
    v3: &mut __m128i,
    v4: &mut __m128i,
    v5: &mut __m128i,
    v6: &mut __m128i,
    v7: &mut __m128i,
) {
    let (d0, d1) = unsafe { load_pair(ptr) };
    let (d2, d3) = unsafe { load_pair(ptr.add(32)) };
    let (d4, d5) = unsafe { load_pair(ptr.add(64)) };
    let (d6, d7) = unsafe { load_pair(ptr.add(96)) };

    *v0 = unsafe { aes_round(*v0, d0) };
    *v1 = unsafe { aes_round(*v1, d1) };
    *v2 = unsafe { aes_round(*v2, d2) };
    *v3 = unsafe { aes_round(*v3, d3) };
    *v4 = unsafe { aes_round(*v4, d4) };
    *v5 = unsafe { aes_round(*v5, d5) };
    *v6 = unsafe { aes_round(*v6, d6) };
    *v7 = unsafe { aes_round(*v7, d7) };

    *v1 = _mm_xor_si128(*v1, *v0);
    *v3 = _mm_xor_si128(*v3, *v2);
    *v5 = _mm_xor_si128(*v5, *v4);
    *v7 = _mm_xor_si128(*v7, *v6);
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "aes", enable = "avx2")]
pub(crate) unsafe fn hash_bytes_long(ptr: *const u8, len: usize, acc: u64) -> u64 {
    let mut v0 = _mm_set1_epi64x((SECRET[0] ^ acc) as i64);
    let mut v1 = _mm_set1_epi64x((STRIPE_SECRET[0] ^ acc) as i64);
    let mut v2 = _mm_set1_epi64x((SECRET[1] ^ acc) as i64);
    let mut v3 = _mm_set1_epi64x((STRIPE_SECRET[1] ^ acc) as i64);
    let mut v4 = _mm_set1_epi64x((SECRET[2] ^ acc) as i64);
    let mut v5 = _mm_set1_epi64x((STRIPE_SECRET[2] ^ acc) as i64);
    let mut v6 = _mm_set1_epi64x((SECRET[3] ^ acc) as i64);
    let mut v7 = _mm_set1_epi64x((STRIPE_SECRET[3] ^ acc) as i64);

    let mut offset = 0usize;
    while offset + 128 <= len {
        unsafe {
            process_128_block_x86(
                ptr.add(offset),
                &mut v0,
                &mut v1,
                &mut v2,
                &mut v3,
                &mut v4,
                &mut v5,
                &mut v6,
                &mut v7,
            );
        }
        offset += 128;
    }
    unsafe {
        process_128_block_x86(
            ptr.add(len - 128),
            &mut v0,
            &mut v1,
            &mut v2,
            &mut v3,
            &mut v4,
            &mut v5,
            &mut v6,
            &mut v7,
        );
    }

    let sum0 = _mm_xor_si128(_mm_xor_si128(v0, v2), _mm_xor_si128(v4, v6));
    let sum1 = _mm_xor_si128(_mm_xor_si128(v1, v3), _mm_xor_si128(v5, v7));
    let final_vec = _mm_xor_si128(sum0, sum1);

    let lanes = unsafe { lanes(final_vec) };

    finalize_vector(lanes[0], lanes[1], len)
}
