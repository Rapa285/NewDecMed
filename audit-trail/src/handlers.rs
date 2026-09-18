use crate::{
    audit_error::{AuditError, ResultExt},
    constants::ATS_PACKAGE_ID,
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
const DEFAULT_LOGS_PAGE_SIZE: usize = 25;
const MAX_LOGS_PAGE_SIZE: usize = 100;

pub struct Handlers {
    pub audit_tx: Sender<AuditEvent>,
}

impl Handlers {

    pub async fn handle_event(
        State(state): State<Arc<Handlers>>,
        Json(event): Json<EncryptedSignedEvent>, // ← ganti dari SignedEvent
    ) -> Result<impl IntoResponse, AuditError> {

        // ── Step 1: Parse iota_address ────────────────────────────────────────
        let iota_address = IotaAddress::from_str(&event.iota_address)
            .map_err(|_| anyhow!("Invalid Iota Address"))
            .code(StatusCode::BAD_REQUEST)?;

        // ── Step 2: Decode ciphertext dan nonce dari base64 ───────────────────
        let ciphertext = STANDARD
            .decode(&event.ciphertext)
            .map_err(|_| anyhow!("Invalid ciphertext"))
            .code(StatusCode::BAD_REQUEST)?;

        let nonce = STANDARD
            .decode(&event.nonce)
            .map_err(|_| anyhow!("Invalid nonce"))
            .code(StatusCode::BAD_REQUEST)?;

        // ── Step 3: Verifikasi signature atas ciphertext ──────────────────────
        let signature = Utils::construct_signature_from_str(&event.signature)
            .map_err(|_| anyhow!("Invalid signature"))
            .code(StatusCode::BAD_REQUEST)?;

        let intent_message = IntentMessage::new(Intent::personal_message(), ciphertext.clone());

        // signature
        //     .verify_secure(&intent_msg, iota_address, SignatureScheme::ED25519)
        //     .map_err(|_| anyhow::anyhow!(
        //         "verifikasi signature gagal — \
        //          payload mungkin dimanipulasi atau bukan pemilik address {iota_address}"
        //     ))?;

        let _ = signature
            .verify_secure(
                &intent_message,
                iota_address,
                SignatureScheme::ED25519,
            )
            .map_err(|_| anyhow!("Failed to verify signature"))
            .code(StatusCode::UNAUTHORIZED)?;

        println!("[ATS] signature valid untuk address {iota_address}");

        // ── Step 4: Dekripsi AES key dengan private key ATS ───────────────────
        let aes_key = ecies_decrypt_key(&event.enc_aes_key, &Utils::ats_private_key_pem())
            .map_err(|e| anyhow::anyhow!("gagal dekripsi AES key: {e}"))
            .code(StatusCode::BAD_REQUEST)?;

        // ── Step 5: Dekripsi payload ──────────────────────────────────────────
        let plaintext = aes_decrypt(&ciphertext, &aes_key, &nonce)
            .map_err(|e| anyhow::anyhow!("gagal dekripsi payload: {e}"))
            .code(StatusCode::BAD_REQUEST)?;

        // ── Step 6: Deserialize AuditEvent ────────────────────────────────────
        let audit_event: AuditEvent = serde_json::from_slice(&plaintext)
            .map_err(|e| anyhow::anyhow!("gagal parse AuditEvent: {e}"))
            .code(StatusCode::BAD_REQUEST)?;


        // ── Masukkan ke audit queue ───────────────────────────────────────────────
        if let Err(e) = state.audit_tx.send(audit_event).await {
            eprintln!("[audit] gagal masukkan event ke queue: {e}");
        }

        Ok(Json(json!({"status": "success"})))
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
