use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt};
use chrono::Utc;
use uuid::Uuid;
use std::sync::mpsc::Receiver;
use crate::types::{AuditEvent, AuditRecord};
use crate::constants::LOG_FILE_PATH;

pub struct AuditLogger {
    rx: Receiver<AuditEvent>,
    prev_record_hash: Option<String>,
}

impl AuditLogger {
    pub fn new(rx: Receiver<AuditEvent>) -> Self {
        Self {
            rx,
            prev_record_hash: None,
        }
    }

    pub async fn run(mut self) {
        while let Some(event) = self.rx.recv() {

            // 1. Buat AuditRecord menggunakan hash sebelumnya
            let record = match create_audit_record(
                event,
                self.prev_record_hash.clone(),
            ) {
                Ok(record) => record,

                Err(e) => {
                    eprintln!(
                        "[audit] gagal membuat AuditRecord: {e}"
                    );
                    continue;
                }
            };

            // Tulis ke file log
            if let Err(e) = write_audit_record(&record).await {
                eprintln!(
                    "[audit] gagal menulis AuditRecord: {e}"
                );
            }

            // Update hash sebelumnya
            self.prev_record_hash = Some(record.record_hash.clone());
        }
    }
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