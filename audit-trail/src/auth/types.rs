// Auth Types — Fase 2: Event Source Authentication
//
// Referensi:
//   - Putz et al. 2019: non-repudiation via per-source signing
//   - Barabanov & Makrushin 2021: event source auth sebagai security
//     requirement untuk audit logging di microservice architecture
//
// Alur:
//   1. Setiap source generate key pair sendiri saat startup
//   2. Source mendaftarkan public key-nya ke ATS (out-of-band / sekali)
//   3. Setiap event dibungkus dalam SignedEvent sebelum dikirim
//   4. ATS verifikasi signature sebelum event masuk ke LAVA pipeline
//
// Yang ditandatangani: source_id | timestamp_rfc3339 | nonce | payload_json
// Ini menjamin:
//   - Non-repudiation  : hanya source dengan private key yang bisa sign
//   - Replay resistance: nonce unik per event mencegah replay
//   - Integrity        : payload tidak bisa diubah di transit

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Event yang sudah ditandatangani oleh event source.
/// Satu-satunya format yang diterima ATS dari event source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedEvent {
    /// ID unik event source — harus terdaftar di SourceRegistry
    pub source_id: String,
    /// Payload event yang sesungguhnya
    pub payload: serde_json::Value,
    /// Timestamp pembuatan event (UTC ISO-8601)
    pub timestamp: DateTime<Utc>,
    /// Nonce random (hex 16 bytes) — mencegah replay attack
    pub nonce: String,
    /// Tanda tangan Ed25519 atas canonical_message() dalam hex
    pub signature: String,
}

impl SignedEvent {
    /// Bangun canonical message yang ditandatangani.
    /// Urutan field dijamin sama di sisi pengirim dan penerima.
    pub fn canonical_message(&self) -> Result<String, serde_json::Error> {
        let payload_str = serde_json::to_string(&self.payload)?;
        Ok(format!(
            "{}|{}|{}|{}",
            self.source_id,
            self.timestamp.to_rfc3339(),
            self.nonce,
            payload_str,
        ))
    }
}

/// Info source yang terdaftar di SourceRegistry ATS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub source_id: String,
    /// Public key Ed25519 hex — diserahkan saat registrasi
    pub public_key_hex: String,
    pub description: Option<String>,
    pub registered_at: DateTime<Utc>,
}