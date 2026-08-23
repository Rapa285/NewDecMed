use crate::{
    audit_error::AuditError,
    constants::ATS_PACKAGE_ID,
    iota_client::IotaLogClient,
    types::{ApiLogRecord, AuditEvent, GetLogsQueryParams, GetLogsResponse, SignedEvent},
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

    /// `GET /api/logs?cursor=<hex ObjectID>&limit=<n>`
    ///
    /// Returns a page of `IotaLogMetadata` for `LogRecord` objects
    /// published on-chain by the log rotation worker (see
    /// `iota_client::IotaLogClient::publish_metadata` /
    /// `utils::Utils::spawn_log_rotation_worker`).
    ///
    /// `cursor` should be the `next_cursor` returned by the previous
    /// call; omit it to get the first page. `limit` defaults to 25 and
    /// is clamped to 100.
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
