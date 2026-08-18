mod handlers;
mod constants;
mod types;
mod audit_error;
mod macros;
mod utils;

use std::{env, sync::Arc};
use axum::{
    routing::post,
    Router,
};
use tokio::fs;
use handlers::Handlers;
use utils::Utils;

use tokio::sync::mpsc; 
use crate::{
    constants::LOG_DIR,
    types::AuditRecord,
    // utils::{spawn_log_writer_worker,spawn_log_rotation_worker}
};

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     dotenvy::dotenv().ok();

//     // 1. Pastikan folder log tersedia
//     if let Err(e) = fs::create_dir_all(LOG_DIR).await {
//         eprintln!("Peringatan: Gagal membuat folder log: {}", e);
//     }

//     // 2. Buat antrean mpsc (kapasitas 10.000 event)
//     let (tx, rx) = mpsc::channel::<AuditRecord>(10000);

//     // 3. Simpan Sender (tx) ke dalam State Handlers
//     let app_handlers = Arc::new(Handlers {
//         event_queue: tx,
//     });

//     // 4. Jalankan KEDUA worker dari utils.rs
//     Utils::spawn_log_writer_worker(rx); // Membaca dari antrean dan menulis
//     Utils::spawn_log_rotation_worker(); // Melakukan rotasi dan upload berkala

//     // 5. Setup Router Axum
//     let app = Router::new()
//         .route("/api/events", post(Handlers::handle_event))
//         .with_state(app_handlers);

//     let port = env::var("PORT")?;

//     let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
//         .await
//         .unwrap();
    
//     println!("Service berjalan dan mendengarkan di port {}...", port);

//     axum::serve(listener, app).await.unwrap();

//     Ok(())

// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    fs::create_dir_all(LOG_DIR).await.ok();

    // ── Channel 1: AuditRecord (event source → LAVA worker) ──────────────
    // Tidak berubah dari sebelumnya
    let (tx, rx) = mpsc::channel::<AuditRecord>(10_000);

    // ── Channel 2: LogItem (LAVA engine → file writer) ───────────────────
    // UnboundedSender masuk ke LavaEngine, Receiver dibaca oleh file writer
    let (lava_tx, lava_rx) = mpsc::unbounded_channel::<LogItem>();

    // ── Setup LAVA engine ─────────────────────────────────────────────────
    let params = LavaParams {
        a: 5,   // authenticator setiap 5 entries
        b: 10,  // flush ke file setiap 10 items
        c: 50,  // rotate credential setiap 50 entries
        d: 60,  // metronome setiap 60 detik
        e: 25,  // verification anchor setiap 25 entries
    };
    let engine = LavaEngine::new(params.clone(), lava_tx)?;

    // Simpan kedua key ini → kirim ke IOTA (Anda yang handle)
    let _initial_pk = engine.initial_public_key().to_string();
    let _lt_pk      = engine.long_term_public_key().to_string();

    let engine = Arc::new(Mutex::new(engine));

    // ── Setup source registry ─────────────────────────────────────────────
    // Load dari env / config — tambahkan semua source yang diizinkan
    let mut registry = SourceRegistry::new();
    // Contoh — dalam produksi load dari config file atau env:
    // registry.register("web-app-01", &env::var("WEB_APP_PUBKEY")?, None)?;
    let verifier = Arc::new(Mutex::new(EventVerifier::new(registry)));

    // ── Setup Handlers ────────────────────────────────────────────────────
    let app_handlers = Arc::new(Handlers {
        event_queue: tx,
        verifier,
    });

    // ── Spawn workers ─────────────────────────────────────────────────────
    Utils::spawn_log_writer_worker(rx, Arc::clone(&engine));  // ← tambah engine
    Utils::spawn_lava_file_writer(lava_rx, params.b);         // ← worker baru
    Utils::spawn_metronome(Arc::clone(&engine), params.d);    // ← worker baru
    Utils::spawn_log_rotation_worker();                        // tidak berubah

    // ── Router — tidak berubah ────────────────────────────────────────────
    let app = Router::new()
        .route("/api/events", post(Handlers::handle_event))
        .with_state(app_handlers);

    let port = env::var("PORT")?;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    println!("ATS listening on :{port}");
    axum::serve(listener, app).await?;

    Ok(())
}