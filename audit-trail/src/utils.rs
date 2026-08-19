
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use reqwest::Body;

use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::time::Duration;
use tokio::sync::mpsc::Receiver;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::Utc;
use anyhow::Context;
use crate::constants::IPFS_BASE_URL;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    constants::{LOG_ROTATION_INTERVAL_SECS, LOG_FILE_PATH, LOG_DIR}, 
    audit_error::AuditError,
    types::{AuditRecord, UtilIpfsAddResponse, SignedAuditEvent},
    current_fn,
    lava::{engine::LavaEngine, types::{LavaParams, LogItem}},
    iota_client::{IotaLogClient, IotaLogMetadata, LavaParamsMeta},
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

    /// WORKER 2: Rotasi, Upload IPFS, dan Publish ke IOTA
    pub fn spawn_log_rotation_worker(
        iota_node_url: String,       // ← dari env var IOTA_NODE_URL
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                Duration::from_secs(LOG_ROTATION_INTERVAL_SECS)
            );
            let iota_client = IotaLogClient::new(iota_node_url);
            let mut sequence_number: u64 = 0;
            let mut prev_block_id: Option<String> = None;

            loop {
                interval.tick().await;

                if let Ok(metadata) = fs::metadata(LOG_FILE_PATH).await {
                    if metadata.len() == 0 {
                        continue; // file kosong, skip
                    }
                } else {
                    continue;
                }

                let timestamp = Utc::now();
                let temp_file_path = format!(
                    "{}/uploading_{}.log",
                    LOG_DIR,
                    timestamp.timestamp()
                );

                // ── 1. Rename file (atomic) ───────────────────────────────────
                if let Err(e) = fs::rename(LOG_FILE_PATH, &temp_file_path).await {
                    eprintln!("[rotation] gagal rename file log: {e}");
                    continue;
                }
                println!("[rotation] log dirotasi: {temp_file_path}");

                // ── 2. Hitung hash file SEBELUM upload ────────────────────────
                // Ini yang menutup window of vulnerability secara partial:
                // hash tercatat sebelum file dikirim ke mana pun
                let file_hash = match IotaLogClient::hash_file(&temp_file_path).await {
                    Ok(h) => h,
                    Err(e) => {
                        eprintln!("[rotation] gagal hash file: {e}");
                        "unknown".to_string()
                    }
                };

                // ── 3. Upload ke IPFS ─────────────────────────────────────────
                let cid = match Self::add_file_to_ipfs(&temp_file_path).await {
                    Ok(cid) => {
                        println!("[rotation] upload IPFS berhasil. CID: {cid}");
                        cid
                    }
                    Err(e) => {
                        eprintln!("[rotation] gagal upload IPFS: {e:?}");
                        // Tetap lanjut kirim ke IOTA meski IPFS gagal
                        // agar hash file tetap tercatat
                        "ipfs_upload_failed".to_string()
                    }
                };

                // ── 4. Publish ke IOTA ────────────────────────────────────────────
                let iota_metadata = IotaLogMetadata {
                    version: "1.0".to_string(),
                    log_sequence_number: sequence_number,
                    rotation_timestamp: timestamp,
                    ipfs_cid: cid,
                    file_hash,
                    prev_object_id: prev_object_id.clone(),
                };

                match iota_client.publish_metadata(&iota_metadata).await {
                    Ok(result) => {
                        println!(
                            "[rotation] IOTA OK — Object ID: {} | TX: {}",
                            result.object_id, result.tx_digest
                        );
                        // Simpan object_id untuk file log berikutnya (chain of custody)
                        prev_object_id = Some(result.object_id);
                    }
                    Err(e) => {
                        eprintln!("[rotation] gagal publish ke IOTA: {e:?}");
                    }
                }

                // ── 5. Hapus file lokal setelah semua berhasil ────────────────
                if let Err(e) = fs::remove_file(&temp_file_path).await {
                    eprintln!("[rotation] gagal hapus file temp: {e}");
                }
            }
        });
    }

    pub fn verify_and_extract_event(signed_payload: SignedAuditEvent) -> Result<AuditEvent> {
        
        // 1. Decode Public Key dari Hex ke bentuk byte (32 byte)
        let pubkey_bytes = hex::decode(&signed_payload.public_key)
            .context("Format public key bukan hex yang valid")?;
        
        let pubkey_array: [u8; 32] = pubkey_bytes.try_into()
            .map_err(|_| anyhow!("Ukuran public key salah (harus 32 byte)"))?;
        
        let verifying_key = VerifyingKey::from_bytes(&pubkey_array)
            .context("Gagal memuat VerifyingKey dari byte yang diberikan")?;

        // 2. Decode Signature dari Hex ke bentuk byte (64 byte)
        let sig_bytes = hex::decode(&signed_payload.signature)
            .context("Format signature bukan hex yang valid")?;
        
        let sig_array: [u8; 64] = sig_bytes.try_into()
            .map_err(|_| anyhow!("Ukuran signature salah (harus 64 byte)"))?;
        
        let signature = Signature::from_bytes(&sig_array);

        // 3. Serialize ulang data event ke bentuk bytes (JSON)
        let payload_bytes = serde_json::to_vec(&signed_payload.event)
            .context("Gagal melakukan serialize pada event payload")?;

        // 4. Verifikasi signature terhadap payload bytes
        if verifying_key.verify(&payload_bytes, &signature).is_err() {
            // Jika tanda tangan salah atau data dimanipulasi, hentikan dengan error
            bail!("Digital signature tidak valid! Payload mungkin telah dimanipulasi atau public key salah.");
        }

        // 5. Jika lolos verifikasi, ekstrak dan kembalikan AuditEvent aslinya
        Ok(signed_payload.event)
    }

    /// HASH-CHAIN

    pub fn calculate_record_hash(
        record_id: &Uuid,
        timestamp: &DateTime<Utc>,
        prev_record_hash: Option<&str>,
        event: &AuditEvent,
    ) -> Result<String, serde_json::Error> {
        let hash_input = serde_json::json!({
            "record_id": record_id,
            "timestamp": timestamp,
            "prev_record_hash": prev_record_hash,
            "event": event,
        });

        let serialized = serde_json::to_vec(&hash_input)?;

        let mut hasher = Sha256::new();
        hasher.update(serialized);

        Ok(hex::encode(hasher.finalize()))
    }

    pub fn create_audit_record(
        event: AuditEvent,
        prev_record_hash: Option<String>,
    ) -> Result<AuditRecord, serde_json::Error> {
        let record_id = Uuid::now_v7();
        let timestamp = Utc::now();

        let record_hash = calculate_record_hash(
            &record_id,
            &timestamp,
            prev_record_hash.as_deref(),
            &event,
        )?;

        Ok(AuditRecord {
            record_id,
            timestamp,
            prev_record_hash,
            record_hash,
            event,
        })
    }

    pub fn verify_record(record: &AuditRecord) -> Result<bool, serde_json::Error> {
        let calculated_hash = calculate_record_hash(
            &record.record_id,
            &record.timestamp,
            record.prev_record_hash.as_deref(),
            &record.event,
        )?;

        Ok(calculated_hash == record.record_hash)
    }

    pub fn verify_chain(
        records: &[AuditRecord],
    ) -> Result<bool, serde_json::Error> {
        let mut expected_previous_hash: Option<String> = None;

        for record in records {
            // Periksa hubungan dengan record sebelumnya.
            if record.prev_record_hash != expected_previous_hash {
                return Ok(false);
            }

            // Hitung ulang hash record.
            let calculated_hash = calculate_record_hash(
                &record.record_id,
                &record.timestamp,
                record.prev_record_hash.as_deref(),
                &record.event,
            )?;

            // Periksa integritas record.
            if calculated_hash != record.record_hash {
                return Ok(false);
            }

            expected_previous_hash = Some(record.record_hash.clone());
        }

        Ok(true)
    }

    pub fn create_audit_batch(
        records: &[AuditRecord],
        file_hash: String,
        ipfs_cid: String,
    ) -> Option<AuditBatch> {
        if records.is_empty() {
            return None;
        }

        let first = records.first()?;
        let last = records.last()?;

        Some(AuditBatch {
            batch_id: Uuid::new_v4(),

            start_time: first.timestamp,
            end_time: last.timestamp,

            record_count: records.len() as u64,

            first_record_id: first.record_id,
            last_record_id: last.record_id,

            first_record_hash: first.record_hash.clone(),
            final_record_hash: last.record_hash.clone(),

            file_hash,
            ipfs_cid,
        })
    }
}