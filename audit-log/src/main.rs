mod handlers;
mod constants;
mod types;
mod audit_error;
mod macros;
mod utils;
mod iota_client;
mod iota_utils;
mod audit;
mod crypto;

use std::{
    env,
    sync::{Arc, atomic::{AtomicUsize, Ordering}},
};
use axum::{
    routing::{get, post},
    Router,
};
use tokio::fs;
use handlers::Handlers;
use utils::Utils;
use tokio::sync::mpsc;
use crate::{
    constants::{LOG_DIR, ALS_PACKAGE_ID},
    types::EncryptedSignedEvent,
    audit::AuditLogger,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // 1. Pastikan folder log tersedia
    if let Err(e) = fs::create_dir_all(LOG_DIR).await {
        eprintln!("Peringatan: Gagal membuat folder log: {}", e);
    }

    // 2. Channel mpsc untuk event masuk (kapasitas 10.000)
    let (tx, rx) = mpsc::channel::<EncryptedSignedEvent>(10_000);

    // 3. Counter record (dipakai writer + rotator)
    let record_counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let writer_counter  = Arc::clone(&record_counter);
    let rotator_counter = Arc::clone(&record_counter);

    // 4. State Handlers
    let app_handlers = Arc::new(Handlers { audit_tx: tx });

    // 5. Jalankan AuditLogger (tulis ke file)
    let audit_logger = AuditLogger::new(rx, writer_counter);
    tokio::spawn(audit_logger.run());

    // 6. Jalankan worker rotasi + upload IPFS + publish IOTA
    Utils::spawn_log_rotation_worker(ALS_PACKAGE_ID.to_string(), rotator_counter);

    // 7. Router
    //
    //   POST /api/events          ← terima event dari klien (butuh Handlers state)
    //   GET  /api/logs/metadata   ← daftar log on-chain (paginasi offset)
    //   GET  /api/logs/record     ← fetch + verify + decrypt satu file log dari IPFS
    //
    let app = Router::new()
        // --- route yang butuh Handlers state ---
        .route("/api/events", post(Handlers::handle_event))
        .with_state(app_handlers)
        // --- route stateless (tidak butuh Handlers) ---
        .route("/api/logs/metadata", get(Handlers::get_logs_metadata))
        .route("/api/logs/record",   get(Handlers::get_record_by_cid));

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await?;

    println!("[ALS] berjalan di port {port}");
    axum::serve(listener, app).await?;

    Ok(())
}