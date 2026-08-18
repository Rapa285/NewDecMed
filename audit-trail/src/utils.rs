
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use reqwest::Body;

use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::time::Duration;
use tokio::sync::mpsc::Receiver;
use chrono::Utc;
use anyhow::Context;
use crate::constants::IPFS_BASE_URL;

use crate::{
    constants::{LOG_ROTATION_INTERVAL_SECS, LOG_FILE_PATH, LOG_DIR}, 
    audit_error::AuditError,
    types::{AuditRecord, UtilIpfsAddResponse},
    current_fn,
};

use lava_ats::lava::{
    engine::LavaEngine,
    types::LogItem,
};

pub struct Utils {}

impl Utils {

    pub async fn add_file_to_ipfs(file_path: &str) -> Result<String, AuditError> {

        let file = File::open(file_path)
            .await
            .context(current_fn!())?;

        let stream = FramedRead::new(file, BytesCodec::new());
        let body = Body::wrap_stream(stream);

        let file_part = reqwest::multipart::Part::stream(body)
            .file_name("audit_trail.log")
            .mime_str("text/plain")
            .context(current_fn!())?;

        let form = reqwest::multipart::Form::new().part("file", file_part);
        let req_client = reqwest::Client::new();
        
        let res = req_client
            .post(format!("{}/add", IPFS_BASE_URL))
            .multipart(form)
            .send()
            .await
            .context(current_fn!())?;

        println!("Respons dari IPFS: {:#?}", res);

        // --- PENGECEKAN STATUS (Tetap dipertahankan) ---
        if !res.status().is_success() {
            let status_code = res.status();
            let error_text = res.text().await.unwrap_or_else(|_| "Gagal membaca body error".to_string());
            
            return Err(anyhow::anyhow!("IPFS Server Error ({}): {}", status_code, error_text).into());
        }
        // -----------------------------------------------

        // 1. Parsing response menjadi JSON dinamis (tanpa struct khusus)
        let res_parsed: serde_json::Value = res
            .json()
            .await
            .context(current_fn!())?;

        println!("Respons_parsed dari IPFS: {:#?}", res_parsed);

        // 2. Ambil nilai CID secara manual. 
        // Catatan: API standar IPFS biasanya menggunakan field "Hash" atau "cid". 
        // Sesuaikan dengan format kembalian gateway Anda.
        let cid = res_parsed["cid"] // Atau ganti menjadi res_parsed["cid"]
            .as_str()
            .unwrap_or("unknown_cid") // Nilai default jika field tidak ditemukan
            .to_string();

        Ok(cid)
    }

    /// WORKER 1A: Penulis Log — sekarang melalui LAVA engine
    ///
    /// Sebelumnya: rx → serialize → tulis file
    /// Sekarang  : rx → LavaEngine::process_event() → LogItem ke lava_tx
    ///
    /// File tidak lagi ditulis di sini — itu tugas Worker 1B (spawn_lava_file_writer)
    pub fn spawn_log_writer_worker(
        mut rx: Receiver<AuditRecord>,
        engine: Arc<Mutex<LavaEngine>>,  // ← parameter baru
    ) {
        tokio::spawn(async move {
            while let Some(record) = rx.recv().await {
                // Konversi AuditRecord → serde_json::Value untuk LAVA
                // AuditRecord tetap utuh — LAVA membungkusnya sebagai payload
                // di dalam LogEntry, hash chain dan signature diurus oleh engine
                let payload = match serde_json::to_value(&record) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("[lava-worker] gagal serialisasi record: {e}");
                        continue;
                    }
                };

                // Proses lewat LAVA engine
                // Engine akan:
                //   1. Advance hash chain
                //   2. Buat LogEntry dengan hash baru
                //   3. Kirim LogEntry (+ Authenticator / CredentialUpdate jika waktunya)
                //      ke lava_tx yang sudah di-setup di main.rs
                let mut eng = engine.lock().await;
                if let Err(e) = eng.process_event(payload) {
                    eprintln!("[lava-worker] engine error: {e}");
                    // Jangan continue — record hilang lebih berbahaya dari log error
                    // Pertimbangkan untuk crash / alert di produksi
                }
            }

            eprintln!("[lava-worker] channel ditutup, worker berhenti");
        });
    }

    /// WORKER 1B: LAVA File Writer
    ///
    /// Membaca LogItem dari lava_rx (output LAVA engine) dan
    /// menulis ke file.log sebagai NDJSON, dengan batching b item per flush.
    ///
    /// Ini menggantikan logika file write yang sebelumnya ada di Worker 1A.
    pub fn spawn_lava_file_writer(
        mut lava_rx: UnboundedReceiver<LogItem>,
        batch_size: u64,  // parameter b dari LavaParams
    ) {
        tokio::spawn(async move {
            let mut buffer: Vec<String> = Vec::new();

            while let Some(item) = lava_rx.recv().await {
                // Serialize LogItem ke JSON
                match serde_json::to_string(&item) {
                    Ok(json_str) => buffer.push(json_str),
                    Err(e) => {
                        eprintln!("[lava-writer] gagal serialisasi LogItem: {e}");
                        continue;
                    }
                }

                // Flush ke file jika buffer sudah penuh (parameter b)
                if buffer.len() as u64 >= batch_size {
                    Self::flush_buffer(&mut buffer).await;
                }
            }

            // Channel ditutup — flush sisa buffer
            if !buffer.is_empty() {
                Self::flush_buffer(&mut buffer).await;
            }

            eprintln!("[lava-writer] lava channel ditutup, writer berhenti");
        });
    }

    /// WORKER 1C: Metronome Timer
    ///
    /// Inject dummy entry ke LAVA engine setiap d detik.
    /// Mencegah truncation attack — verifier tahu harus ada
    /// minimal 1 entry per interval d.
    pub fn spawn_metronome(
        engine: Arc<Mutex<LavaEngine>>,
        interval_secs: u64,  // parameter d dari LavaParams
    ) {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(
                tokio::time::Duration::from_secs(interval_secs)
            );
            ticker.tick().await; // skip tick pertama

            loop {
                ticker.tick().await;
                let mut eng = engine.lock().await;
                if let Err(e) = eng.inject_metronome() {
                    eprintln!("[metronome] error: {e}");
                }
            }
        });
    }

    /// Helper: flush buffer ke file.log
    async fn flush_buffer(buffer: &mut Vec<String>) {
        let mut file = match OpenOptions::new()
            .create(true)
            .append(true)
            .open(LOG_FILE_PATH)
            .await
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[lava-writer] gagal buka file log: {e}");
                return;
            }
        };

        for line in buffer.drain(..) {
            let with_newline = line + "\n";
            if let Err(e) = file.write_all(with_newline.as_bytes()).await {
                eprintln!("[lava-writer] gagal tulis ke file: {e}");
            }
        }

        if let Err(e) = file.flush().await {
            eprintln!("[lava-writer] gagal flush file: {e}");
        }
    }

    /// WORKER 2: Rotasi dan Upload ke IPFS
    pub fn spawn_log_rotation_worker() {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(LOG_ROTATION_INTERVAL_SECS));

            loop {
                interval.tick().await;

                if let Ok(metadata) = fs::metadata(LOG_FILE_PATH).await {
                    if metadata.len() > 0 {
                        let timestamp = Utc::now().timestamp();
                        let temp_file_path = format!("{}/uploading_{}.log", LOG_DIR, timestamp);

                        match fs::rename(LOG_FILE_PATH, &temp_file_path).await {
                            Ok(_) => {
                                println!("Menyiapkan log untuk di-upload: {}", temp_file_path);

                                match Self::add_file_to_ipfs(&temp_file_path).await {
                                    Ok(cid) => {
                                        println!("Berhasil upload ke IPFS. CID: {}", cid);
                                        if let Err(e) = fs::remove_file(&temp_file_path).await {
                                            eprintln!("Gagal menghapus file lokal: {}", e);
                                        }
                                    }
                                    Err(e) => eprintln!("Gagal upload ke IPFS: {:?}", e),
                                }
                            }
                            Err(e) => eprintln!("Gagal merename file log: {}", e),
                        }
                    }
                }
            }
        });
    }

    
}