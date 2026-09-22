use crate::{
    audit_error::{AuditError, ResultExt},
    constants::{ALS_PACKAGE_ID, DEFAULT_LOGS_PAGE_SIZE, MAX_LOGS_PAGE_SIZE, IPFS_GATEWAY_BASE_URL},
    iota_client::IotaLogClient,
    types::{
        ApiLogRecord, AuditRecord, GetLogsQueryParams, GetLogsResponse,
        GetRecordByCidQueryParams, GetRecordByCidResponse,
        EncryptedSignedEvent, DecryptedRecordResult
    },
    utils::Utils,
};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};

use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use anyhow::anyhow;

pub struct Handlers {
    pub audit_tx: Sender<EncryptedSignedEvent>,
}

impl Handlers {

    // =========================================================================
    // POST /api/events — Terima event dari klien
    // =========================================================================

    pub async fn handle_event(
        State(state): State<Arc<Handlers>>,
        Json(event): Json<EncryptedSignedEvent>,
    ) -> Result<impl IntoResponse, AuditError> {
        println!("[audit] menerima event baru");

        // Verifikasi signature — tolak jika tidak valid
        if let Err(e) = Utils::verify_event_signature(&event) {
            eprintln!("[audit] event ditolak — signature tidak valid: {e}");
            return Ok(Json(json!({
                "status": "error",
                "message": "signature verification failed"
            })));
        }

        // Kirim ke channel untuk diproses AuditLogger
        if let Err(e) = state.audit_tx.send(event).await {
            eprintln!("[audit] gagal kirim ke queue: {e}");
            return Ok(Json(json!({"status": "error", "message": "internal error"})));
        }

        Ok(Json(json!({"status": "success"})))
    }

    // =========================================================================
    // GET /api/logs/metadata — Daftar metadata log yang tersimpan on-chain
    // =========================================================================

    pub async fn get_logs_metadata(
        Query(params): Query<GetLogsQueryParams>,
    ) -> Result<impl IntoResponse, AuditError> {
        let limit = params
            .limit
            .unwrap_or(DEFAULT_LOGS_PAGE_SIZE)
            .clamp(1, MAX_LOGS_PAGE_SIZE);

        let cursor = params.cursor;

        let client = IotaLogClient::new(ALS_PACKAGE_ID)?;

        // Coba Move function view dulu, fallback ke direct object read
        let page = match client.list_log_records(cursor, limit).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[get_logs_metadata] Move call gagal ({e}), coba fallback...");
                client.list_log_records_via_object(cursor, limit).await?
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

    // =========================================================================
    // GET /api/logs/record?cid=<cid>
    //
    // Alur:
    //   1. Fetch file .log dari IPFS berdasarkan CID
    //   2. Parse setiap baris JSON → AuditRecord
    //   3. Verifikasi hash chain (record_hash tiap baris)
    //   4. Verifikasi IOTA signature pada setiap EncryptedSignedEvent
    //   5. Dekripsi payload dengan private key ALS (ECIES + AES-GCM)
    //   6. Kembalikan list AuditEvent yang sudah terdekripsi
    // =========================================================================

    pub async fn get_record_by_cid(
        Query(params): Query<GetRecordByCidQueryParams>,
    ) -> Result<impl IntoResponse, AuditError> {

        let cid = params.cid.trim().to_string();
        if cid.is_empty() {
            return Err(AuditError::Anyhow {
                source: anyhow!("parameter 'cid' tidak boleh kosong"),
                code: StatusCode::BAD_REQUEST,
            });
        }

        // ── Step 1: Fetch file log dari IPFS ─────────────────────────────────

        let raw_content = Self::fetch_from_ipfs(&cid).await?;

        // ── Step 2: Parse baris per baris (format JSONL) ──────────────────────

        let records: Vec<AuditRecord> = Self::parse_jsonl(&raw_content)?;

        if records.is_empty() {
            return Ok(Json(GetRecordByCidResponse {
                cid: cid.clone(),
                total_records: 0,
                chain_valid: true,
                results: vec![],
            }));
        }

        // ── Step 3: Verifikasi hash chain ─────────────────────────────────────

        let chain_valid = Self::verify_hash_chain(&records);

        // ── Step 4 + 5: Verifikasi signature & dekripsi setiap record ─────────

        let results = Self::verify_and_decrypt_records(records)?;

        Ok(Json(GetRecordByCidResponse {
            cid,
            total_records: results.len() as u64,
            chain_valid,
            results,
        }))
    }

    // ── Helper: fetch dari IPFS ───────────────────────────────────────────────

    async fn fetch_from_ipfs(cid: &str) -> Result<String, AuditError> {
        let url = format!("{}/ipfs/{}", IPFS_GATEWAY_BASE_URL, cid);

        eprintln!("[ipfs] fetching: {url}");

        let client = reqwest::Client::builder()
            // Timeout 60 detik, file log bisa besar
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| AuditError::Anyhow {
                source: anyhow!("gagal build HTTP client: {e}"),
                code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| AuditError::Anyhow {
                source: anyhow!("gagal fetch dari IPFS (url={url}): {e}"),
                code: StatusCode::BAD_GATEWAY,
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AuditError::Anyhow {
                source: anyhow!("IPFS gateway error {status}: {body}"),
                code: StatusCode::BAD_GATEWAY,
            });
        }

        let text = resp
            .text()
            .await
            .map_err(|e| AuditError::Anyhow {
                source: anyhow!("gagal baca body IPFS: {e}"),
                code: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

        eprintln!("[ipfs] berhasil fetch {} bytes", text.len());
        Ok(text)
    }

    // ── Helper: parse JSONL ───────────────────────────────────────────────────

    fn parse_jsonl(raw: &str) -> Result<Vec<AuditRecord>, AuditError> {
        let mut records = Vec::new();
        let mut errors = 0usize;

        for (line_num, line) in raw.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            match serde_json::from_str::<AuditRecord>(line) {
                Ok(r) => records.push(r),
                Err(e) => {
                    eprintln!("[parse] skip baris {}: {e}", line_num + 1);
                    errors += 1;
                }
            }
        }

        eprintln!(
            "[parse] {} record berhasil di-parse, {} baris dilewati",
            records.len(),
            errors
        );

        Ok(records)
    }

    // ── Helper: verifikasi hash chain ─────────────────────────────────────────
    //
    // Setiap record menyimpan record_hash yang dihitung dari:
    //   (record_id, timestamp, prev_record_hash, ciphertext, iota_address)
    // Kita hitung ulang dan bandingkan.

    fn verify_hash_chain(records: &[AuditRecord]) -> bool {
        let mut all_valid = true;

        for (i, record) in records.iter().enumerate() {
            match Utils::verify_record(record) {
                Ok(valid) => {
                    if !valid {
                        eprintln!(
                            "[chain] record {} (id={}) hash TIDAK cocok",
                            i,
                            record.record_id
                        );
                        all_valid = false;
                    }
                }
                Err(e) => {
                    eprintln!("[chain] error verifikasi record {}: {e}", i);
                    all_valid = false;
                }
            }
        }

        if all_valid {
            eprintln!("[chain] semua {} record valid ✓", records.len());
        }

        all_valid
    }

    // ── Helper: verifikasi signature + dekripsi ───────────────────────────────

    fn verify_and_decrypt_records(
        records: Vec<AuditRecord>,
    ) -> Result<Vec<DecryptedRecordResult>, AuditError> {
        let mut results = Vec::with_capacity(records.len());

        for (i, record) in records.into_iter().enumerate() {
            let record_id = record.record_id.to_string();
            let timestamp  = record.timestamp;
            let prev_hash  = record.prev_record_hash.clone();
            let record_hash = record.record_hash.clone();

            // Step 4: verifikasi IOTA signature
            let sig_valid = match Utils::verify_event_signature(&record.event) {
                Ok(()) => true,
                Err(e) => {
                    eprintln!("[sig] record {} signature gagal: {e}", i);
                    false
                }
            };

            // Step 5: dekripsi payload (hanya jika signature valid)
            let (decrypted_event, decrypt_error) = if sig_valid {
                match Utils::decrypt_event(&record.event) {
                    Ok(event) => (Some(event), None),
                    Err(e) => {
                        let msg = format!("gagal dekripsi: {e}");
                        eprintln!("[decrypt] record {}: {}", i, msg);
                        (None, Some(msg))
                    }
                }
            } else {
                (None, Some("signature tidak valid, dekripsi dilewati".to_string()))
            };

            results.push(DecryptedRecordResult {
                record_id,
                timestamp,
                prev_record_hash: prev_hash,
                record_hash,
                iota_address: record.event.iota_address.clone(),
                signature_valid: sig_valid,
                decrypt_error,
                event: decrypted_event,
            });
        }

        Ok(results)
    }
}