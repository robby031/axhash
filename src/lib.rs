#![no_std]

mod bytes;
mod constants;
mod hasher;
mod math;
mod memory;

pub use hasher::{AxBuildHasher, AxHasher, axhash, axhash_of, axhash_of_seeded, axhash_seeded};

#[cfg(test)]
mod tests {
    use super::{AxHasher, axhash, axhash_of, axhash_of_seeded, axhash_seeded};
    use core::hash::{Hash, Hasher};

    #[derive(Hash)]
    struct DemoRecord {
        id: u64,
        shard: u32,
        flags: u32,
    }

    #[test]
    fn hash_is_deterministic_for_bytes() {
        let data = b"axhash regression seed";
        let a = axhash_seeded(data, 0x1234_5678_9abc_def0);
        let b = axhash_seeded(data, 0x1234_5678_9abc_def0);
        assert_eq!(a, b);
    }

    #[test]
    fn hash_changes_when_seed_changes() {
        let data = b"same payload different seed";
        let a = axhash_seeded(data, 1);
        let b = axhash_seeded(data, 2);
        assert_ne!(a, b);
    }

    #[test]
    fn hash_trait_path_is_deterministic() {
        let record = DemoRecord {
            id: 42,
            shard: 7,
            flags: 3,
        };
        let a = axhash_of_seeded(&record, 0xdead_beef);
        let b = axhash_of_seeded(&record, 0xdead_beef);
        assert_eq!(a, b);
    }

    #[test]
    fn primitive_writes_produce_a_stable_finish() {
        let mut hasher = AxHasher::new_with_seed(0x4444);
        hasher.write_u64(0x0102_0304_0506_0708);
        hasher.write_u32(0xaabb_ccdd);
        hasher.write_u16(0xeeff);
        hasher.write_u8(0x11);
        let value = hasher.finish();
        assert_ne!(value, 0);
    }

    #[test]
    fn test_axhash_default_seed() {
        let data = b"default seed test";
        let a = axhash(data);
        let b = axhash_seeded(data, 0); // Varian axhash harus identik dengan seed 0
        assert_eq!(a, b);
    }

    #[test]
    fn test_axhash_of_default_seed() {
        let record = DemoRecord {
            id: 1,
            shard: 2,
            flags: 3,
        };
        let a = axhash_of(&record);
        let b = axhash_of_seeded(&record, 0); // Varian axhash_of harus identik dengan seed 0
        assert_eq!(a, b);
    }
}
