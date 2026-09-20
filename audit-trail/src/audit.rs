use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Receiver; // ← tokio, bukan std
use chrono::Utc;
use uuid::Uuid;
use crate::types::{AuditEvent, AuditRecord, EncryptedSignedEvent};
use crate::constants::LOG_FILE_PATH;
use crate::utils::Utils;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

pub struct AuditLogger {
    rx: Receiver<EncryptedSignedEvent>,
    record_counter: Arc<AtomicUsize>,
    prev_record_hash: Option<String>,
}

impl AuditLogger {
    pub fn new(rx: Receiver<EncryptedSignedEvent>, record_counter: Arc<AtomicUsize>) -> Self {
        Self {
            rx,
            record_counter,
            prev_record_hash: None,
        }
    }

    pub async fn run(mut self) {
        while let Some(encrypted_event) = self.rx.recv().await {
            let record = match create_audit_record(
                encrypted_event,
                self.prev_record_hash.clone(),
            ) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[audit] gagal buat record: {e}");
                    continue;
                }
            };

            if let Err(e) = write_audit_record(&record).await {
                eprintln!("[audit] gagal tulis record: {e}");
            }

            self.record_counter.fetch_add(1, Ordering::Relaxed);
            self.prev_record_hash = Some(record.record_hash.clone());
        }
    }
}

pub fn create_audit_record(
    event: EncryptedSignedEvent,
    prev_record_hash: Option<String>,
) -> Result<AuditRecord, serde_json::Error> {
    let record_id = Uuid::now_v7();
    let timestamp = Utc::now();

    // Delegasikan ke Utils::calculate_record_hash (single source of truth)
    let record_hash = Utils::calculate_record_hash(
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

pub async fn write_audit_record(
    record: &AuditRecord,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let json = serde_json::to_string(record)?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE_PATH)
        .await?;

    file.write_all(json.as_bytes()).await?;
    file.write_all(b"\n").await?;

    Ok(())
}