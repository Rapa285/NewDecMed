use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── LAVA Parameters ────────────────────────────────────────────────────────────
// Sesuai paper Bajramovic et al. 2023, Table 1.
// Semua parameter minimum 1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LavaParams {
    /// Jumlah log entries per authenticator (authentication efficiency)
    pub a: u64,
    /// Jumlah log entries yang dikirim dalam satu pesan / flush (communication efficiency)
    pub b: u64,
    /// Jumlah log entries per credential rotation (forward integrity)
    pub c: u64,
    /// Interval metronome dalam detik (truncation resistance)
    pub d: u64,
    /// Jumlah log entries yang bisa di-skip saat fast-forward verification
    pub e: u64,
}

impl Default for LavaParams {
    fn default() -> Self {
        Self {
            a: 5,
            b: 10,
            c: 50,
            d: 60,
            e: 25,
        }
    }
}

impl LavaParams {
    pub fn validate(&self) -> crate::lava::error::LavaResult<()> {
        use crate::lava::error::LavaError;
        if self.a == 0 || self.b == 0 || self.c == 0 || self.d == 0 || self.e == 0 {
            return Err(LavaError::InvalidParams(
                "semua parameter LAVA harus >= 1".into(),
            ));
        }
        Ok(())
    }
}

// ── Entry Types ────────────────────────────────────────────────────────────────
// Mengikuti notasi paper LAVA Section 4.
// Setiap item yang masuk ke output queue adalah salah satu dari tipe ini.

/// Log entry biasa — dikirim oleh event source (tipe "l" di paper)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Nomor urut global, di-increment tiap entry biasa
    pub index: u64,
    /// Hash chain saat ini: H(prev_hash || index || data)
    pub hash: String,
    /// Timestamp saat entry dibuat
    pub timestamp: DateTime<Utc>,
    /// Payload dari event source
    pub data: serde_json::Value,
}

/// Authenticator — tanda tangan digital atas hash chain saat ini (tipe "Z" di paper)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Authenticator {
    /// Index entry terakhir yang di-cover authenticator ini
    pub entry_index: u64,
    /// Hash chain pada saat authenticator dibuat
    pub hash: String,
    /// Tanda tangan digital: Sign(sk, hash)
    pub signature: String,
}

/// Credential update — public key baru yang ditandatangani key lama (tipe "A" di paper)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialUpdate {
    /// Index entry saat rotation terjadi
    pub entry_index: u64,
    /// Public key baru dalam format hex
    pub new_public_key: String,
    /// Tanda tangan atas new_public_key menggunakan private key lama
    pub signature: String,
}

/// Verification anchor — data untuk fast-forward verification (tipe "E" di paper)
/// Ditulis ke log tiap e entries agar verifier bisa skip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationAnchor {
    /// Index entry saat anchor dibuat
    pub entry_index: u64,
    /// Hash chain saat ini
    pub hash: String,
    /// Public key A yang aktif saat ini (hex)
    pub current_public_key: String,
    /// Tanda tangan anchor oleh long-term key E
    pub signature: String,
}

/// Metronome entry — dummy event untuk truncation resistance (tipe "l" subtype metronome)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetronomeEntry {
    pub index: u64,
    pub hash: String,
    pub timestamp: DateTime<Utc>,
}

// ── Log Item ──────────────────────────────────────────────────────────────────
// Enum yang merepresentasikan semua kemungkinan item dalam output queue / file.log

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LogItem {
    /// Log entry biasa
    Entry(LogEntry),
    /// Authenticator
    Authenticator(Authenticator),
    /// Credential update
    CredentialUpdate(CredentialUpdate),
    /// Verification anchor untuk fast-forward
    VerificationAnchor(VerificationAnchor),
    /// Metronome dummy event
    Metronome(MetronomeEntry),
}