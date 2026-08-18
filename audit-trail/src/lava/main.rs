// Contoh penggunaan LAVA engine secara end-to-end:
//   1. Engine menerima events
//   2. Metronome berjalan di background
//   3. Writer menyimpan ke file.log
//   4. Verifier memvalidasi hasilnya

use std::{path::PathBuf, sync::Arc};
use tokio::sync::{mpsc, Mutex};

use lava_ats::lava::{
    engine::LavaEngine,
    metronome,
    types::LavaParams,
    verifier::Verifier,
    writer,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let params = LavaParams {
        a: 3,   // authenticator setiap 3 entries
        b: 5,   // flush ke disk setiap 5 items
        c: 10,  // rotate credential setiap 10 entries
        d: 5,   // metronome setiap 5 detik
        e: 9,   // verification anchor setiap 9 entries
    };

    let log_path = PathBuf::from("/tmp/audit.log");

    // ── Setup pipeline ────────────────────────────────────────────────────────
    let (tx, rx) = mpsc::unbounded_channel();
    let mut engine = LavaEngine::new(params.clone(), tx)?;

    // Simpan keys — ini yang harus dikirim ke IOTA sebelum log dimulai
    let initial_pk = engine.initial_public_key().to_string();
    let lt_pk = engine.long_term_public_key().to_string();

    println!("=== LAVA ATS dimulai ===");
    println!("Initial public key (simpan ke IOTA): {}", &initial_pk[..16]);
    println!("Long-term public key (simpan ke IOTA): {}", &lt_pk[..16]);
    println!("Params: a={} b={} c={} d={}s e={}", params.a, params.b, params.c, params.d, params.e);

    // Wrap engine dalam Arc<Mutex> untuk shared ownership dengan metronome
    let engine = Arc::new(Mutex::new(engine));

    // Spawn metronome timer
    let metro_handle = metronome::spawn(Arc::clone(&engine), params.d);

    // Spawn writer task
    let log_path_clone = log_path.clone();
    let batch_size = params.b;
    let writer_handle = tokio::spawn(async move {
        writer::run_writer(rx, log_path_clone, batch_size).await
    });

    // ── Simulasi events masuk ─────────────────────────────────────────────────
    println!("\n--- Memproses 15 events ---");
    for i in 0u64..15 {
        let event = serde_json::json!({
            "source": "web-app-01",
            "action": "user.login",
            "user_id": format!("user-{}", i % 3),
            "ip": "192.168.1.1",
            "seq": i,
        });

        {
            let mut eng = engine.lock().await;
            eng.process_event(event)?;
        }
        println!("  event {} diproses", i);

        // Simulasi jeda antar event
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // ── Shutdown graceful ─────────────────────────────────────────────────────
    metro_handle.abort();

    // Drop engine untuk close channel → writer akan flush dan selesai
    drop(engine);

    let total_written = writer_handle.await??;
    println!("\n--- Selesai: {} items ditulis ke {:?} ---", total_written, log_path);

    // ── Verifikasi ────────────────────────────────────────────────────────────
    println!("\n=== Memulai verifikasi log ===");
    let verifier = Verifier::new(params, initial_pk, lt_pk);
    match verifier.verify_file(&log_path).await {
        Ok(report) => {
            println!("✓ Log VALID");
            println!("  Total items    : {}", report.total_items);
            println!("  Log entries    : {}", report.total_entries);
            println!("  Authenticators : {}", report.total_authenticators);
            println!("  Cred updates   : {}", report.total_credential_updates);
            println!("  Metronomes     : {}", report.total_metronomes);
        }
        Err(e) => {
            eprintln!("✗ Log TIDAK VALID: {e}");
            std::process::exit(1);
        }
    }

    Ok(())
}