mod handlers;
mod constants;
mod types;
mod audit_error;
mod macros;
mod utils;
mod iota_client;
mod iota_utils;
mod audit;

use std::{env, sync::Arc};
use axum::{
    routing::{get, post},
    Router,
};
use tokio::fs;
use tower_http::cors::CorsLayer;
use handlers::Handlers;
use utils::Utils;
use tokio::sync::mpsc; 
use crate::{
    constants::{LOG_DIR,IOTA_URL,ATS_PACKAGE_ID},
    types::AuditEvent,
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
    let (tx, rx) = mpsc::channel::<AuditEvent>(10000);

    // 3. Simpan Sender (tx) ke dalam State Handlers
    let app_handlers = Arc::new(Handlers {
        audit_tx: tx,
    });
    let audit_logger = AuditLogger::new(rx);
    tokio::spawn(audit_logger.run());

    // 4. Jalankan worker dari utils.rs
    Utils::spawn_log_rotation_worker(ATS_PACKAGE_ID.to_string()); // Melakukan rotasi dan upload berkala

    // 5. CORS — diperlukan agar client desktop (Tauri webview / dev server
    //    di localhost:1420) bisa memanggil endpoint ini. Persempit origin
    //    di production jika perlu (mis. hanya izinkan "tauri://localhost").
    let cors = CorsLayer::permissive();

    // 6. Setup Router Axum
    let app = Router::new()
        .route("/api/events", post(Handlers::handle_event))
        .with_state(app_handlers)
        // GET /api/logs tidak butuh Handlers state, jadi dipasang
        // terpisah dari router ber-state di atas.
        .route("/api/logs", get(Handlers::get_logs))
        .layer(cors);

    let port = env::var("PORT")?;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    
    println!("Service berjalan dan mendengarkan di port {}...", port);

    axum::serve(listener, app).await.unwrap();

    Ok(())

}
