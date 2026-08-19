use reqwest::StatusCode;
use crate::{
    constants::LOG_FILE_PATH,
    // types::{ExecuteTxResponse, ReserveGasResponse, SuccessResponse, UtilIpfsAddResponse},
    types::{AuditRecord, AuditEvent},
    auth::{verifier::EventVerifier, types::SignedEvent},
    utils::Utils
};
use uuid::Uuid;

use axum::{extract::State, Json, response::IntoResponse};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use chrono::Utc;

pub struct Handlers {
    pub event_queue: tokio::sync::mpsc::Sender<AuditRecord>,
    pub verifier: Arc<Mutex<EventVerifier>>,
}

impl Handlers {

    // pub async fn new_audit_record(data: String) -> Result<String, AuditError> {

    //     // berikan ID
    //     let uuid = Uuid::now_v7();

    //     // add ke ipfs
    //     let audit_record = AuditRecord {
    //         id: uuid,
    //         data,
    //     };

    //     let cid = Utils::add_and_pin_to_ipfs(audit_record).await?;

    //     // add cid ke IOTA
    //     // uuid
    //     // ipfs cid
        

    //     // add indexing ke postgres
    //     // uuid
    //     // ts
    //     // event type
    //     // actor
    //     // ipfs cid
    //     // iota id

    //     Ok(String::from("fungsi berhasil"))
    // }

    // pub async fn collect_audit_event(
    //     // Json(payload):Json<HandlerCollectAuditEventPayload>
    //     payload : String,
    // ) -> Result<String, AuditError> {
        
    //     // diberikan id

    //     // diberikan timestamp

    //     // daftarkan ke IOTA

    //     Ok(String::from("fungsi berhasil"))
    // }

    // pub async fn delete_audit_record(cid: String) -> Result<String, AuditError> {

    //     Ok()
    // }

    // pub async fn handle_event(
    //     State(_state): State<Arc<Handlers>>, 
    //     Json(payload): Json<AuditEvent>,    
    // ) -> impl IntoResponse {
        
    //     println!("Payload diterima: {:#?}", payload);
        
    //     let new_record_id = Uuid::now_v7();
    //     let current_timestamp = Utc::now();
    //     let calculated_prev_hash = Some("hash_sementara".to_string());

    //     let audit_record = AuditRecord {
    //         record_id: new_record_id,
    //         timestamp: current_timestamp,
    //         prev_record_hash: calculated_prev_hash,
    //         event: payload, 
    //     };

    //     let log_line = match serde_json::to_string(&audit_record) {
    //         Ok(json_str) => json_str + "\n",
    //         Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Serialisasi error: {}", e)).into_response(),
    //     };

    //     // Langsung menggunakan LOG_FILE_PATH yang konstan
    //     let mut file: tokio::fs::File = match OpenOptions::new()
    //         .create(true) 
    //         .append(true) 
    //         .open(LOG_FILE_PATH)
    //         .await 
    //     {
    //         Ok(f) => f,
    //         Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Akses file error: {}", e)).into_response(),
    //     };

    //     if let Err(e) = file.write_all(log_line.as_bytes()).await {
    //         eprintln!("Gagal menulis ke file log: {}", e);
    //         // return (StatusCode::INTERNAL_SERVER_ERROR, "Gagal menyimpan log").into_response();
    //     }

    //     let response_body = json!({
    //         "status": "success",
    //         "record_id": new_record_id,
    //     });

    //     (StatusCode::CREATED, Json(response_body)).into_response()
    // }


    pub async fn handle_event(
        State(state): State<Arc<Handlers>>,
        Json(signed): Json<SignedEvent>,
    ) -> impl IntoResponse {

        // ── Fase 2: verifikasi source ─────────────────────────────────────
        // Cek: source terdaftar? nonce belum dipakai? signature valid?
        let audit_event = Utils::verify_and_extract_event(signed);


        // ── Buat AuditRecord — Uuid dan timestamp tetap seperti sebelumnya ─
        // prev_record_hash tidak lagi diisi di sini —
        // hash chain sekarang diurus sepenuhnya oleh LAVA engine
        let audit_record = AuditRecord {
            record_id: Uuid::now_v7(),
            timestamp: Utc::now(),
            prev_record_hash: None,  // LAVA yang akan isi ini
            event: audit_event,
        };

        // ── Kirim ke channel → masuk LAVA pipeline ────────────────────────
        // Tidak ada lagi file write di sini — itu tugas LAVA writer worker
        match state.event_queue.send(audit_record.clone()).await {
            Ok(_) => (
                StatusCode::CREATED,
                Json(json!({
                    "status": "success",
                    "record_id": audit_record.record_id,
                }))
            ).into_response(),
            Err(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "status": "error", "reason": "queue penuh atau worker mati" }))
            ).into_response(),
        }
    }
}