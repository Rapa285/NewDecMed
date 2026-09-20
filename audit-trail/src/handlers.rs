use crate::{
    audit_error::{AuditError, ResultExt},
    constants::{ATS_PACKAGE_ID, DEFAULT_LOGS_PAGE_SIZE, MAX_LOGS_PAGE_SIZE},
    iota_client::IotaLogClient,
    types::{ApiLogRecord, AuditEvent, GetLogsQueryParams, GetLogsResponse, EncryptedSignedEvent},
    utils::Utils,
    crypto::{ecies_decrypt_key, aes_decrypt},

};

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Json},
    http::StatusCode
};

use iota_types::base_types::{IotaAddress,ObjectID};
use shared_crypto::intent::{Intent, IntentMessage};
use iota_types::crypto::{SignatureScheme,Signature, IotaSignature};
use std::str::FromStr;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};

// Default and max page size for GET /api/logs, mirroring the pattern
// used elsewhere in decmed (e.g. patient::get_access_log clamps to 10).
// const DEFAULT_LOGS_PAGE_SIZE: usize = 25;
// const MAX_LOGS_PAGE_SIZE: usize = 100;

pub struct Handlers {
    pub audit_tx: Sender<EncryptedSignedEvent>,
}

impl Handlers {

    pub async fn handle_event(
        State(state): State<Arc<Handlers>>,
        Json(event): Json<EncryptedSignedEvent>,
    ) -> Result<impl IntoResponse, AuditError> {
        println!("menerima event");
        // Hanya verifikasi signature — tidak ada dekripsi
        if let Err(e) = Utils::verify_event_signature(&event) {
            eprintln!("[audit] event ditolak — signature tidak valid: {e}");
            return Ok(Json(json!({
                "status": "error",
                "message": "signature verification failed"
                // Sengaja tidak expose detail error ke pengirim
            })));
        }

        if let Err(e) = state.audit_tx.send(event).await {
            eprintln!("[audit] gagal kirim ke queue: {e}");
            return Ok(Json(json!({"status": "error", "message": "internal error"})));
        }

        Ok(Json(json!({"status": "success"})))
    }

    // pub async fn get_decrypted_logs(
    //     Query(params): Query<GetLogsQueryParams>,
    // ) -> Result<impl IntoResponse, AuditError> {
    //     // Baca dari file log
    //     let records = read_audit_records_from_file().await?;

    //     let decrypted: Vec<serde_json::Value> = records
    //         .iter()
    //         .map(|record| {
    //             // Dekripsi saat audit
    //             match Utils::decrypt_event(&record.encrypted_event) {
    //                 Ok(event) => json!({
    //                     "record_id": record.record_id,
    //                     "timestamp": record.timestamp,
    //                     "prev_record_hash": record.prev_record_hash,
    //                     "record_hash": record.record_hash,
    //                     "event": event, // plaintext hanya di response, tidak di storage
    //                 }),
    //                 Err(e) => json!({
    //                     "record_id": record.record_id,
    //                     "error": format!("gagal dekripsi: {e}"),
    //                 }),
    //             }
    //         })
    //         .collect();

    //     Ok(Json(json!({ "data": decrypted })))
    // }

    pub async fn get_logs_metadata(
        Query(params): Query<GetLogsQueryParams>,
    ) -> Result<impl IntoResponse, AuditError> {
        let limit = params
            .limit
            .unwrap_or(DEFAULT_LOGS_PAGE_SIZE)
            .clamp(1, MAX_LOGS_PAGE_SIZE);

        let cursor = params.cursor; // Option<u64>

        let client = IotaLogClient::new(ATS_PACKAGE_ID)?;

        // Coba lewat Move function dulu; jika gagal, fallback ke object read
        let page = client
            .list_log_records(cursor, limit)
            .await
            .or_else(|_| {
                // Jalankan async block — perlu runtime handle
                // Gunakan tokio::task::block_in_place jika diperlukan,
                // atau langsung return error dan biarkan caller pakai fallback
                Err(anyhow::anyhow!("fallback triggered"))
            });

        let page = match page {
            Ok(p) => p,
            Err(_) => {
                // Fallback: baca langsung dari object content
                client
                    .list_log_records_via_object(cursor, limit)
                    .await?
            }
        };

        let data: Vec<ApiLogRecord> = page
            .records
            .into_iter()
            .map(|r| ApiLogRecord {
                index: r.index,
                json_data: r.json_data,
                metadata: r.metadata,
            })
            .collect();

        Ok(Json(GetLogsResponse {
            data,
            total: page.total,
            cursor: cursor.unwrap_or(0),
            has_next_page: page.has_next_page,
        }))
    }
}
