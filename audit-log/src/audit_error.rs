use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuditError {
    #[error("Error (code {code}): {source}")]
    Anyhow {
        #[source]
        source: anyhow::Error,
        code: StatusCode,
    },
}

impl IntoResponse for AuditError {
    fn into_response(self) -> Response {
        let (error_message, code) = match self {
            AuditError::Anyhow { source, code } => (format!("{:?}", source), code),
        };

        let error_response = json!({
            "status_code": code.as_u16(),
            "error": error_message,
        });

        (code, Json(error_response)).into_response()
    }
}

impl serde::Serialize for AuditError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&format!("{:?}", self))
    }
}

impl From<anyhow::Error> for AuditError {
    fn from(value: anyhow::Error) -> Self {
        AuditError::Anyhow {
            source: value,
            code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub trait ResultExt<T> {
    fn code(self, code: StatusCode) -> Result<T, AuditError>;
}

impl<T> ResultExt<T> for anyhow::Result<T> {
    fn code(self, code: StatusCode) -> Result<T, AuditError> {
        self.map_err(|e| AuditError::Anyhow { source: e, code })
    }
}
