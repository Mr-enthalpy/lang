//! Deterministic build fingerprinting (FNV-1a, 64-bit).
//!
//! Shared helper used both by physical source-content fingerprints
//! (`discovery.rs`) and by static build-graph package fingerprints / cache keys
//! (`build.rs`).
//!
//! This is a **deterministic build fingerprint only**:
//!
//! - not cryptographic;
//! - not a stable ABI identity;
//! - not a cache-validity proof across compiler versions unless the cache
//!   format version is included in the hashed material.

const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const PRIME: u64 = 0x0000_0100_0000_01b3;

/// Streaming FNV-1a-64 accumulator.
///
/// Use [`Fnv1a64::write_field`] / [`Fnv1a64::write_str_field`] when folding
/// multiple inputs into one digest so that concatenation is unambiguous
/// (`"ab" + "c"` must not collide with `"a" + "bc"`).
#[derive(Clone, Debug)]
pub struct Fnv1a64 {
    hash: u64,
}

impl Fnv1a64 {
    pub fn new() -> Self {
        Self { hash: OFFSET_BASIS }
    }

    /// Fold raw bytes into the digest with no framing.
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.hash ^= u64::from(byte);
            self.hash = self.hash.wrapping_mul(PRIME);
        }
    }

    /// Fold a length-prefixed field so neighbouring fields cannot blur together.
    pub fn write_field(&mut self, bytes: &[u8]) {
        self.write_bytes(&(bytes.len() as u64).to_le_bytes());
        self.write_bytes(bytes);
    }

    /// Fold a length-prefixed UTF-8 field.
    pub fn write_str_field(&mut self, value: &str) {
        self.write_field(value.as_bytes());
    }

    /// Finish into a stable 16-character lowercase hex digest.
    pub fn finish_hex(&self) -> String {
        format!("{:016x}", self.hash)
    }
}

impl Default for Fnv1a64 {
    fn default() -> Self {
        Self::new()
    }
}

/// One-shot FNV-1a-64 hex digest of raw bytes (no field framing).
pub fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hasher = Fnv1a64::new();
    hasher.write_bytes(bytes);
    hasher.finish_hex()
}
