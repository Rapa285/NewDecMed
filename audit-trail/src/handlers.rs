use crate::{
    types::{AuditEvent, SignedEvent},
    utils::Utils
};

use axum::{extract::State, Json, response::IntoResponse};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub struct Handlers {
    pub audit_tx: Sender<AuditEvent>,
}

impl Handlers {

    pub async fn handle_event(
        State(state): State<Arc<Handlers>>,
        Json(signed): Json<SignedEvent>,
    ) -> impl IntoResponse {

        // ── Fase 1: verifikasi source ─────────────────────────────────
        let audit_event = match Utils::verify_and_extract_event(signed) {
            Ok(event) => event,
            Err(e) => {
                eprintln!("[audit] gagal verifikasi event: {e}");
                return Json(json!({"status": "error", "message": format!("{e}")}));
            }
        };

        // ── Fase 2: masukkan event ke audit queue ─────────────────
        if let Err(e) = state.audit_tx.send(audit_event).await {
            eprintln!(
                "[audit] gagal memasukkan event ke queue: {e}"
            );
        }

        // response endpoint
        Json(json!({"status": "success"}))
    }
}