use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("source '{source_id}' tidak terdaftar di registry")]
    UnknownSource { source_id: String },

    #[error("signature tidak valid dari source '{source_id}'")]
    InvalidSignature { source_id: String },

    #[error("payload tidak valid: {0}")]
    InvalidPayload(String),

    #[error("source '{source_id}' sudah terdaftar")]
    DuplicateSource { source_id: String },

    #[error("kriptografi gagal: {0}")]
    CryptoError(String),

    #[error("serialisasi gagal: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type AuthResult<T> = Result<T, AuthError>;