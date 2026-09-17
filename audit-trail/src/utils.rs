use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use reqwest::Body;

use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::time::Duration;
use anyhow::{anyhow, bail, Result, Context};
use sha2::{Sha256, Digest};

use crate::{
    constants::{LOG_ROTATION_INTERVAL_SECS, LOG_FILE_PATH, LOG_DIR, IPFS_BASE_URL}, 
    audit_error::AuditError,
    types::{AuditRecord, SignedEvent, AuditEvent},
    current_fn,
    iota_client::{IotaLogClient, IotaLogMetadata},
};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Revisi
use iota_types::base_types::IotaAddress;
use iota_types::crypto::{Signature as IotaSignature, SignatureScheme};
use shared_crypto::intent::{Intent, IntentMessage};
use std::str::FromStr;

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

        // println!("Respons dari IPFS: {:#?}", res);

        if !res.status().is_success() {
            let status_code = res.status();
            let error_text = res.text().await.unwrap_or_else(|_| "Gagal membaca body error".to_string());
            
            return Err(anyhow::anyhow!("IPFS Server Error ({}): {}", status_code, error_text).into());
        }

        let res_parsed: serde_json::Value = res
            .json()
            .await
            .context(current_fn!())?;

        // println!("Respons_parsed dari IPFS: {:#?}", res_parsed);

        let cid = res_parsed["cid"]
            .as_str()
            .unwrap_or("unknown_cid")
            .to_string();

        Ok(cid)
    }

    /// WORKER: Rotasi, Upload IPFS, dan Publish ke IOTA
    pub fn spawn_log_rotation_worker(
        package_id: String,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                Duration::from_secs(LOG_ROTATION_INTERVAL_SECS)
            );

            let iota_client = IotaLogClient::new(&package_id)
                .expect("Gagal membuat IotaLogClient");

            let mut sequence_number: u64 = 0;
            let mut prev_tx_digest: Option<String> = None;

            loop {
                interval.tick().await;

                if let Ok(metadata) = fs::metadata(LOG_FILE_PATH).await {
                    if metadata.len() == 0 {
                        continue;
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

                if let Err(e) = fs::rename(LOG_FILE_PATH, &temp_file_path).await {
                    eprintln!("[rotation] gagal rename file log: {e}");
                    continue;
                }
                println!("[rotation] log dirotasi: {temp_file_path}");

                let file_hash = match IotaLogClient::hash_file(&temp_file_path).await {
                    Ok(h) => h,
                    Err(e) => {
                        eprintln!("[rotation] gagal hash file: {e}");
                        "unknown".to_string()
                    }
                };

                let cid = match Self::add_file_to_ipfs(&temp_file_path).await {
                    Ok(cid) => {
                        println!("[rotation] upload IPFS berhasil. CID: {cid}");
                        cid
                    }
                    Err(e) => {
                        eprintln!("[rotation] gagal upload IPFS: {e:?}");
                        "ipfs_upload_failed".to_string()
                    }
                };

                let iota_metadata = IotaLogMetadata {
                    version: "1.0".to_string(),
                    log_sequence_number: sequence_number,
                    rotation_timestamp: timestamp,
                    ipfs_cid: cid,
                    file_hash,
                    first_record_hash: String::new(),
                    final_record_hash: String::new(),
                    record_count: 0,
                    prev_tx_digest: prev_tx_digest.clone(),
                };

                match iota_client.publish_metadata(&iota_metadata).await {
                    Ok(result) => {
                        println!(
                            "[rotation] IOTA OK — Object ID: {} | TX: {}",
                            result.object_id, result.tx_digest
                        );
                        prev_tx_digest = Some(result.tx_digest);
                        sequence_number += 1;
                    }
                    Err(e) => {
                        eprintln!("[rotation] gagal publish ke IOTA: {e:?}");
                    }
                }

                if let Err(e) = fs::remove_file(&temp_file_path).await {
                    eprintln!("[rotation] gagal hapus file temp: {e}");
                }
            }
        });
    }

    pub fn verify_and_extract_event(
        signed_payload: SignedEvent,
    ) -> Result<AuditEvent> {

        // 1. Parse iota_address
        let iota_address = IotaAddress::from_str(&signed_payload.iota_address)
            .context("iota_address tidak valid")?;

        // 2. Decode signature (base64 → IotaSignature)
        let signature = IotaSignature::decode_base64(&signed_payload.signature)
            .map_err(|e| anyhow!("gagal decode signature: {e}"))?;

        // 3. Reconstruct IntentMessage dari payload
        //    Harus identik dengan yang di-sign di sisi client
        let intent_msg = IntentMessage::new(
            Intent::personal_message(),
            signed_payload.payload.as_bytes().to_vec(),
        );

        // 4. Verifikasi: signature + iota_address + intent_message
        //    verify_secure memastikan public key dalam signature
        //    sesuai dengan iota_address → tidak perlu binding terpisah!
        signature
            .verify_secure(&intent_msg, iota_address, SignatureScheme::ED25519)
            .map_err(|_| anyhow!("signature tidak valid atau bukan pemilik address"))?;

        // 5. Parse AuditEvent dari payload
        let audit_event: AuditEvent = serde_json::from_str(&signed_payload.payload)
            .context("gagal parse payload menjadi AuditEvent")?;

        Ok(audit_event)
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

    pub fn verify_record(record: &AuditRecord) -> Result<bool, serde_json::Error> {
        let calculated_hash = Self::calculate_record_hash(
            &record.record_id,
            &record.timestamp,
            record.prev_record_hash.as_deref(),
            &record.event,
        )?;

        Ok(calculated_hash == record.record_hash)
    }
}