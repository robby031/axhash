use crate::hasher::AxHasher;
use core::hash::Hasher;

#[test]
fn write_does_not_corrupt_state() {
    let seed = 0x1234_5678;
    let mut h1 = AxHasher::new_with_seed(seed);
    h1.write_u8(0x01);
    h1.write_u8(0x02);
    let after_first = h1.finish();

    let mut h2 = AxHasher::new_with_seed(seed);
    h2.write_u8(0x01);
    h2.write_u8(0x02);
    let after_second = h2.finish();

    assert_eq!(after_first, after_second, "finish() mutated internal state");
}

#[test]
fn u8_sequence_equals_u16() {
    let seed = 0xABCD;
    let mut h1 = AxHasher::new_with_seed(seed);
    h1.write_u8(0x01);
    h1.write_u8(0x02);

    let mut h2 = AxHasher::new_with_seed(seed);
    h2.write_u16(0x0201);

    assert_eq!(h1.finish(), h2.finish(), "u8 sequence != u16 equivalence");
}

#[test]
fn u16_sequence_equals_u32() {
    let seed = 0xBEEF;
    let mut h1 = AxHasher::new_with_seed(seed);
    h1.write_u16(0x0201);
    h1.write_u16(0x0403);

    let mut h2 = AxHasher::new_with_seed(seed);
    h2.write_u32(0x04030201);

    assert_eq!(h1.finish(), h2.finish(), "u16 sequence != u32 equivalence");
}

#[test]
fn u32_sequence_vs_u64_is_predictable() {
    let seed = 0xCAFE;
    let mut h1 = AxHasher::new_with_seed(seed);
    h1.write_u32(0x04030201);
    h1.write_u32(0x08070605);

    let mut h2 = AxHasher::new_with_seed(seed);
    h2.write_u64(0x0807060504030201);

    assert_ne!(
        h1.finish(),
        h2.finish(),
        "u32 sequence unexpectedly matched u64 direct path"
    );
}

#[test]
fn bytes_vs_u16_is_predictable() {
    let seed = 0x1111;
    let mut h1 = AxHasher::new_with_seed(seed);
    h1.write(&[0x01, 0x02]);

    let mut h2 = AxHasher::new_with_seed(seed);
    h2.write_u16(0x0201);

    assert_ne!(
        h1.finish(),
        h2.finish(),
        "bytes write unexpectedly matched u16 sponge path"
    );
}

#[test]
fn sponge_flush_edge_cases() {
    let seed = 0x9999;

    let mut h1 = AxHasher::new_with_seed(seed);
    h1.write(b"");
    let empty_write = h1.finish();

    let h2 = AxHasher::new_with_seed(seed);
    let no_write = h2.finish();

    assert_eq!(empty_write, no_write, "empty write changed hash");

    let mut h3 = AxHasher::new_with_seed(seed);
    h3.write_u8(0x01);
    h3.write(&[0x02, 0x03]);
    h3.write_u16(0x0504);
    let interleaved = h3.finish();

    let mut h4 = AxHasher::new_with_seed(seed);
    h4.write_u8(0x01);
    h4.write(&[0x02, 0x03]);
    h4.write_u16(0x0504);
    assert_eq!(interleaved, h4.finish(), "interleaved writes non-deterministic");
}
