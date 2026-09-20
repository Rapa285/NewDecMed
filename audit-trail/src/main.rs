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
    constants::{LOG_DIR,ATS_PACKAGE_ID},
    types::{AuditEvent,EncryptedSignedEvent},
    audit::AuditLogger,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // 1. Pastikan folder log tersedia
    if let Err(e) = fs::create_dir_all(LOG_DIR).await {
        eprintln!("Peringatan: Gagal membuat folder log: {}", e);
    }

    // 2. Buat antrean mpsc (kapasitas 10.000 event)
    let (tx, rx) = mpsc::channel::<EncryptedSignedEvent>(10000);

    // Buat counter yang bisa dibagikan ke beberapa thread
    let record_counter = Arc::new(AtomicUsize::new(0));

    // Clone untuk diberikan ke fungsi Writer
    let writer_counter = Arc::clone(&record_counter);

    // Clone untuk diberikan ke fungsi Rotasi (loop rotasi)
    let rotator_counter = Arc::clone(&record_counter);

    // 3. Simpan Sender (tx) ke dalam State Handlers
    let app_handlers = Arc::new(Handlers {
        audit_tx: tx,
    });
    
    let audit_logger = AuditLogger::new(rx,writer_counter);
    tokio::spawn(audit_logger.run());

    // 4. Jalankan worker dari utils.rs
    Utils::spawn_log_rotation_worker(ATS_PACKAGE_ID.to_string(),rotator_counter); // Melakukan rotasi dan upload berkala


    // 6. Setup Router Axum
    let app = Router::new()
        .route("/api/events", post(Handlers::handle_event))
        .with_state(app_handlers)
        // GET /api/logs tidak butuh Handlers state, jadi dipasang
        // terpisah dari router ber-state di atas.
        .route("/api/get-logs-metadata", get(Handlers::get_logs_metadata));

    let port = env::var("PORT")?;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    
    println!("Service berjalan dan mendengarkan di port {}...", port);

    axum::serve(listener, app).await.unwrap();

    Ok(())

}
