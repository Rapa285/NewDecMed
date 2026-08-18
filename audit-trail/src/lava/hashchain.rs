// Hash Chain Engine
// Implementasi dari LAVA Section 4 — setiap entry diikat ke entry sebelumnya
// melalui: h_i = H(h_{i-1} || i || entry_data)
// Ini menjamin bahwa modifikasi / penghapusan entri manapun akan
// memutus rantai dan terdeteksi saat verifikasi.

use sha2::{Digest, Sha256};

/// Genesis hash — titik awal rantai, semua nol
pub const GENESIS_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

pub struct HashChain {
    current: String,
}

impl HashChain {
    /// Buat hash chain baru dimulai dari genesis hash
    pub fn new() -> Self {
        Self {
            current: GENESIS_HASH.to_string(),
        }
    }

    /// Lanjutkan dari hash yang sudah ada (untuk recovery / resume)
    pub fn from_existing(hash: String) -> Self {
        Self { current: hash }
    }

    /// Hitung hash berikutnya dan update state internal.
    /// Formula: H(prev_hash || index.to_string() || data)
    pub fn advance(&mut self, index: u64, data: &str) -> String {
        let next = Self::compute(&self.current, index, data);
        self.current = next.clone();
        next
    }

    /// Hitung hash tanpa mengubah state — untuk verifikasi
    pub fn compute(prev_hash: &str, index: u64, data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(prev_hash.as_bytes());
        hasher.update(index.to_string().as_bytes());
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn current(&self) -> &str {
        &self.current
    }
}

impl Default for HashChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_is_deterministic() {
        let mut c1 = HashChain::new();
        let mut c2 = HashChain::new();
        let h1 = c1.advance(0, "event-a");
        let h2 = c2.advance(0, "event-a");
        assert_eq!(h1, h2, "hash chain harus deterministik");
    }

    #[test]
    fn test_chain_breaks_on_tamper() {
        let mut chain = HashChain::new();
        let h0 = chain.advance(0, "event-a");
        let h1 = chain.advance(1, "event-b");

        // Verifikasi ulang manual: ubah data → hash berbeda
        let tampered = HashChain::compute(&h0, 1, "event-TAMPERED");
        assert_ne!(tampered, h1, "data yang diubah harus menghasilkan hash berbeda");
    }

    #[test]
    fn test_genesis_is_constant() {
        let c = HashChain::new();
        assert_eq!(c.current(), GENESIS_HASH);
    }
}