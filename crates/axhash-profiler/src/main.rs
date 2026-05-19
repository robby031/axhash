use axhash_core::{AxHasher, axhash_of};
use std::hash::Hasher;

#[inline(never)]
fn black_box_u64(x: u64) -> u64 {
    std::hint::black_box(x)
}

#[inline(never)]
fn hash_tiny_4(k4: &[u8; 4]) -> u64 {
    axhash_of(k4)
}

#[inline(never)]
fn hash_tiny_8(k8: &[u8; 8]) -> u64 {
    axhash_of(k8)
}

#[inline(never)]
fn hash_tiny_16(k16: &[u8; 16]) -> u64 {
    axhash_of(k16)
}

#[inline(never)]
fn hash_stream_32_chunks(bytes: &[u8; 32]) -> u64 {
    let mut hasher = AxHasher::new();
    hasher.write(&bytes[0..4]);
    hasher.write(&bytes[4..8]);
    hasher.write(&bytes[8..16]);
    hasher.write(&bytes[16..24]);
    hasher.write(&bytes[24..32]);
    hasher.finish()
}

#[inline(never)]
fn run_tiny_key_latency(iterations: u64) -> u64 {
    let mut acc = 0u64;
    let mut i = 0u64;
    while i < iterations {
        let k4: [u8; 4] = (i as u32).to_le_bytes();
        let k8: [u8; 8] = (i.wrapping_mul(0x9E37_79B9_7F4A_7C15)).to_le_bytes();
        let k16: [u8; 16] = [
            k8[0], k8[1], k8[2], k8[3], k8[4], k8[5], k8[6], k8[7], k4[0], k4[1], k4[2], k4[3], 0,
            1, 2, 3,
        ];

        acc ^= hash_tiny_4(&k4);
        acc ^= hash_tiny_8(&k8);
        acc ^= hash_tiny_16(&k16);
        i += 1;
    }
    black_box_u64(acc)
}

fn pattern_bytes32(seed: u64) -> [u8; 32] {
    let mut out = [0u8; 32];
    let mut v = seed;
    let mul = 0x9E37_79B9_7F4A_7C15u64;
    let add = 0xD1B5_4A32_1C27_3F91u64;
    let mut i = 0usize;
    while i < 32 {
        v = v.wrapping_mul(mul).wrapping_add(add);
        out[i] = v as u8;
        i += 1;
    }
    out
}

#[inline(never)]
fn run_tiny_key_streaming(iterations: u64) -> u64 {
    let mut acc = 0u64;
    let mut i = 0u64;
    while i < iterations {
        let bytes = pattern_bytes32(i);
        acc ^= hash_stream_32_chunks(&bytes);
        i += 1;
    }
    black_box_u64(acc)
}

fn main() {
    let iterations: u64 = 5_000_000_000;

    println!("AxHash profiler helper");
    println!("iterations per scenario: {}", iterations);

    let r1 = run_tiny_key_latency(iterations);
    println!("tiny_key_latency sink={}", r1);

    let r2 = run_tiny_key_streaming(iterations / 4);
    println!("tiny_key_streaming sink={}", r2);
}
