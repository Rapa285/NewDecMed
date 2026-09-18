use crate::{
    audit_error::AuditError,
    constants::ATS_PACKAGE_ID,
    iota_client::IotaLogClient,
    types::{ApiLogRecord, AuditEvent, GetLogsQueryParams, GetLogsResponse, EncryptedSignedEvent},
    utils::Utils,
};

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Json},
};
use iota_types::base_types::ObjectID;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

// Default and max page size for GET /api/logs, mirroring the pattern
// used elsewhere in decmed (e.g. patient::get_access_log clamps to 10).
const DEFAULT_LOGS_PAGE_SIZE: usize = 25;
const MAX_LOGS_PAGE_SIZE: usize = 100;

pub struct Handlers {
    pub audit_tx: Sender<AuditEvent>,
}

impl Handlers {

    pub async fn handle_event(
        State(state): State<Arc<Handlers>>,
        Json(signed): Json<EncryptedSignedEvent>, // ← ganti dari SignedEvent
    ) -> impl IntoResponse {

        // ── Dekripsi, verifikasi signature, ekstrak AuditEvent ───────────────────
        let audit_event = match Utils::decrypt_verify_and_extract(signed) {
            Ok(event) => event,
            Err(e) => {
                eprintln!("[audit] gagal proses event: {e}");
                return Json(json!({
                    "status": "error",
                    "message": format!("{e}")
                }));
            }
        };

        // ── Masukkan ke audit queue ───────────────────────────────────────────────
        if let Err(e) = state.audit_tx.send(audit_event).await {
            eprintln!("[audit] gagal masukkan event ke queue: {e}");
        }

        Json(json!({"status": "success"}))
    }

    pub async fn get_logs(
        Query(params): Query<GetLogsQueryParams>,
    ) -> Result<impl IntoResponse, AuditError> {
        let limit = params
            .limit
            .unwrap_or(DEFAULT_LOGS_PAGE_SIZE)
            .clamp(1, MAX_LOGS_PAGE_SIZE);

        let cursor = params
            .cursor
            .map(|c| {
                ObjectID::from_hex_literal(&c)
                    .map_err(|e| anyhow::anyhow!("cursor tidak valid: {e}"))
            })
            .transpose()?;

        let client = IotaLogClient::new(ATS_PACKAGE_ID)?;
        let page = client.list_log_records(cursor, limit).await?;

        let data = page
            .records
            .into_iter()
            .map(|record| ApiLogRecord {
                object_id: record.object_id.to_string(),
                metadata: record.metadata,
            })
            .collect();

        Ok(Json(GetLogsResponse {
            data,
            next_cursor: page.next_cursor.map(|c| c.to_string()),
            has_next_page: page.has_next_page,
        }))
    }
}
