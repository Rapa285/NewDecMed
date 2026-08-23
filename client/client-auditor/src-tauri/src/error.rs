use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Audit-trail returned an error ({status}): {body}")]
    ServerError { status: u16, body: String },

    #[error("Invalid base URL: {0}")]
    InvalidUrl(String),

    #[error("Failed to parse log entry: {0}")]
    Parse(String),

    #[error("Settings store error: {0}")]
    Store(String),
}

// Tauri commands need errors to be Serialize so they can cross the
// IPC boundary and be handled as rejected promises on the frontend.
impl serde::Serialize for ClientError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
