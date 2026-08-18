use thiserror::Error;

#[derive(Debug, Error)]
pub enum LavaError {
    #[error("parameter tidak valid: {0}")]
    InvalidParams(String),

    #[error("kriptografi gagal: {0}")]
    CryptoError(String),

    #[error("verifikasi gagal pada entry index {index}: {reason}")]
    VerificationFailed { index: u64, reason: String },

    #[error("hash chain putus pada index {index}: expected {expected}, got {got}")]
    HashChainBroken {
        index: u64,
        expected: String,
        got: String,
    },

    #[error("authenticator tidak valid pada entry index {index}")]
    InvalidAuthenticator { index: u64 },

    #[error("credential update tidak valid pada entry index {index}")]
    InvalidCredentialUpdate { index: u64 },

    #[error("truncation terdeteksi: timeout menunggu entry berikutnya")]
    TruncationDetected,

    #[error("log kosong atau tidak dapat dibaca")]
    EmptyLog,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialisasi gagal: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type LavaResult<T> = Result<T, LavaError>;