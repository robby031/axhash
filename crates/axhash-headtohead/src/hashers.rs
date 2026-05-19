use std::hash::{BuildHasher, BuildHasherDefault, Hasher};

pub const SEED: u64 = 0xDEAD_BEEF_CAFE_BABE;

pub struct HashEntry {
    pub name: &'static str,
    pub hash_bytes: fn(&[u8]) -> u64,
}

pub struct StreamEntry {
    pub name: &'static str,
    pub stream: fn(&[u8], usize) -> u64,
}

pub fn hash_entries() -> &'static [HashEntry] {
    &[
        HashEntry { name: "axhash",      hash_bytes: ax_hash },
        HashEntry { name: "xxh3",        hash_bytes: xxh3_hash },
        HashEntry { name: "wyhash",      hash_bytes: wy_hash },
        HashEntry { name: "ahash",       hash_bytes: ahash_hash },
        HashEntry { name: "fxhash",      hash_bytes: fx_hash },
        HashEntry { name: "siphash-1-3", hash_bytes: sip_hash },
        HashEntry { name: "highwayhash", hash_bytes: hwy_hash },
    ]
}

pub fn stream_entries() -> &'static [StreamEntry] {
    &[
        StreamEntry { name: "axhash",      stream: ax_stream },
        StreamEntry { name: "xxh3",        stream: xxh3_stream },
        StreamEntry { name: "wyhash",      stream: wy_stream },
        StreamEntry { name: "ahash",       stream: ahash_stream },
        StreamEntry { name: "fxhash",      stream: fx_stream },
        StreamEntry { name: "siphash-1-3", stream: sip_stream },
        StreamEntry { name: "highwayhash", stream: hwy_stream },
    ]
}

fn ax_hash(data: &[u8]) -> u64 {
    axhash::axhash_seeded(data, SEED)
}

fn xxh3_hash(data: &[u8]) -> u64 {
    xxhash_rust::xxh3::xxh3_64_with_seed(data, SEED)
}

fn wy_hash(data: &[u8]) -> u64 {
    wyhash::wyhash(data, SEED)
}

fn ahash_hash(data: &[u8]) -> u64 {
    let mut h = ahash::AHasher::default();
    h.write(data);
    h.finish()
}

fn fx_hash(data: &[u8]) -> u64 {
    let mut h = rustc_hash::FxHasher::default();
    h.write(data);
    h.finish()
}

fn sip_hash(data: &[u8]) -> u64 {
    let mut h = siphasher::sip::SipHasher13::new_with_keys(SEED, SEED >> 1);
    h.write(data);
    h.finish()
}

fn hwy_hash(data: &[u8]) -> u64 {
    use highway::{HighwayHash, HighwayHasher, Key};
    let mut h = HighwayHasher::new(Key([SEED, SEED >> 1, SEED >> 2, SEED >> 3]));
    h.append(data);
    h.finalize64()
}

fn ax_stream(data: &[u8], chunk: usize) -> u64 {
    use std::hash::Hasher;
    let mut h = axhash::AxHasher::new_with_seed(SEED);
    for c in data.chunks(chunk) {
        h.write(std::hint::black_box(c));
    }
    h.finish()
}

fn xxh3_stream(data: &[u8], chunk: usize) -> u64 {
    use std::hash::Hasher;
    let mut h = xxhash_rust::xxh3::Xxh3::with_seed(SEED);
    for c in data.chunks(chunk) {
        h.write(std::hint::black_box(c));
    }
    h.finish()
}

fn wy_stream(data: &[u8], chunk: usize) -> u64 {
    use std::hash::Hasher;
    let mut h = wyhash::WyHash::with_seed(SEED);
    for c in data.chunks(chunk) {
        h.write(std::hint::black_box(c));
    }
    h.finish()
}

fn ahash_stream(data: &[u8], chunk: usize) -> u64 {
    let mut h = ahash::AHasher::default();
    for c in data.chunks(chunk) {
        h.write(std::hint::black_box(c));
    }
    h.finish()
}

fn fx_stream(data: &[u8], chunk: usize) -> u64 {
    let mut h = rustc_hash::FxHasher::default();
    for c in data.chunks(chunk) {
        h.write(std::hint::black_box(c));
    }
    h.finish()
}

fn sip_stream(data: &[u8], chunk: usize) -> u64 {
    let mut h = siphasher::sip::SipHasher13::new_with_keys(SEED, SEED >> 1);
    for c in data.chunks(chunk) {
        h.write(std::hint::black_box(c));
    }
    h.finish()
}

fn hwy_stream(data: &[u8], chunk: usize) -> u64 {
    use highway::{HighwayHash, HighwayHasher, Key};
    let mut h = HighwayHasher::new(Key([SEED, SEED >> 1, SEED >> 2, SEED >> 3]));
    for c in data.chunks(chunk) {
        h.append(std::hint::black_box(c));
    }
    h.finalize64()
}

pub struct HwyStdHasher(Vec<u8>);

impl Hasher for HwyStdHasher {
    fn write(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }
    fn finish(&self) -> u64 {
        use highway::{HighwayHash, HighwayHasher, Key};
        let mut h = HighwayHasher::new(Key([SEED, 0, 0, 0]));
        h.append(&self.0);
        h.finalize64()
    }
}

#[derive(Clone, Default)]
pub struct HwyBuildHasher;

impl BuildHasher for HwyBuildHasher {
    type Hasher = HwyStdHasher;
    fn build_hasher(&self) -> HwyStdHasher {
        HwyStdHasher(Vec::with_capacity(32))
    }
}

pub type AxBH = axhash::AxBuildHasher;
pub type Xxh3BH = BuildHasherDefault<xxhash_rust::xxh3::Xxh3>;
pub type WyBH = BuildHasherDefault<wyhash::WyHash>;
pub type AHashBH = ahash::RandomState;
pub type FxBH = rustc_hash::FxBuildHasher;
pub type SipBH = BuildHasherDefault<siphasher::sip::SipHasher13>;
