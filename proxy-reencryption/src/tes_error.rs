use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TesError {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl serde::Serialize for TesError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&format!("{:?}", self))
    }
}

impl IntoResponse for TesError {
    fn into_response(self) -> Response {
        // Customize status code and message
        let status = StatusCode::INTERNAL_SERVER_ERROR;
        let body = json!({
            "error": format!("{:?}", self) // or use structured data
        });

        (status, Json(body)).into_response()
    }
}
