// src/ats/queue.rs

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::fs::{self, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use uuid::Uuid;

const QUEUE_FILE: &str = "ats_queue.jsonl";
const RETRY_BASE_SECS: u64 = 5;
const RETRY_MAX_SECS: u64 = 300;
const MAX_ATTEMPTS_BEFORE_SKIP: u32 = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueEntry {
    pub id: String,
    /// JSON dari SignedAuditEvent yang sudah ditandatangani dengan IOTA keypair.
    /// Langsung bisa dikirim ke ATS server tanpa proses ulang.
    pub signed_payload: String,
    pub label: String,
    pub created_at: String,
    pub attempt_count: u32,
}

#[derive(Clone)]
pub struct AtsQueue {
    file_path: PathBuf,
}

impl AtsQueue {
    pub fn new(queue_dir: &str) -> Self {
        Self {
            file_path: PathBuf::from(queue_dir).join(QUEUE_FILE),
        }
    }

    pub async fn push(&self, entry: QueueEntry) -> Result<(), String> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("gagal buat direktori queue: {e}"))?;
        }

        let mut line = serde_json::to_string(&entry)
            .map_err(|e| format!("gagal serialize QueueEntry: {e}"))?;
        line.push('\n');

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .await
            .map_err(|e| format!("gagal buka file queue: {e}"))?;

        file.write_all(line.as_bytes())
            .await
            .map_err(|e| format!("gagal tulis ke file queue: {e}"))?;

        Ok(())
    }

    pub async fn read_all(&self) -> Vec<QueueEntry> {
        let file = match fs::File::open(&self.file_path).await {
            Ok(f) => f,
            Err(_) => return vec![],
        };

        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut entries = Vec::new();

        while let Ok(Some(line)) = lines.next_line().await {
            let line = line.trim().to_string();
            if line.is_empty() {
                continue;
            }
            match serde_json::from_str::<QueueEntry>(&line) {
                Ok(entry) => entries.push(entry),
                Err(e) => eprintln!("[ATS Queue] skip baris rusak: {e} | baris: {line}"),
            }
        }

        entries
    }

    pub async fn rewrite(&self, remaining: &[QueueEntry]) -> Result<(), String> {
        if remaining.is_empty() {
            let _ = fs::remove_file(&self.file_path).await;
            return Ok(());
        }

        let tmp_path = self.file_path.with_extension("jsonl.tmp");

        let mut content = String::new();
        for entry in remaining {
            let line = serde_json::to_string(entry)
                .map_err(|e| format!("gagal serialize entry saat rewrite: {e}"))?;
            content.push_str(&line);
            content.push('\n');
        }

        fs::write(&tmp_path, content)
            .await
            .map_err(|e| format!("gagal tulis file tmp: {e}"))?;

        fs::rename(&tmp_path, &self.file_path)
            .await
            .map_err(|e| format!("gagal rename tmp ke queue: {e}"))?;

        Ok(())
    }
}

pub fn new_queue_entry(signed_payload: String, label: &str) -> QueueEntry {
    QueueEntry {
        id: Uuid::new_v4().to_string(),
        signed_payload,
        label: label.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        attempt_count: 0,
    }
}

pub fn spawn_retry_worker(queue: AtsQueue, ats_endpoint: &'static str) {
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let mut consecutive_failures: u32 = 0;

        loop {
            let entries = queue.read_all().await;

            if entries.is_empty() {
                tokio::time::sleep(Duration::from_secs(RETRY_BASE_SECS)).await;
                consecutive_failures = 0;
                continue;
            }

            println!(
                "[ATS Worker] {} entry pending di queue, mencoba kirim...",
                entries.len()
            );

            let mut remaining: Vec<QueueEntry> = Vec::new();
            let mut any_success = false;

            for mut entry in entries {
                if entry.attempt_count >= MAX_ATTEMPTS_BEFORE_SKIP {
                    eprintln!(
                        "[ATS Worker] entry {} sudah {} kali gagal, skip sementara (label: {})",
                        entry.id, entry.attempt_count, entry.label
                    );
                    remaining.push(entry);
                    continue;
                }

                // signed_payload sudah berisi JSON SignedAuditEvent yang siap kirim
                let result = client
                    .post(ats_endpoint)
                    .header("Content-Type", "application/json")
                    .body(entry.signed_payload.clone())
                    .send()
                    .await;

                match result {
                    Ok(res) if res.status().is_success() => {
                        println!(
                            "[ATS Worker] ✓ entry {} berhasil dikirim (label: {}, attempt: {})",
                            entry.id, entry.label, entry.attempt_count + 1
                        );
                        any_success = true;
                        // Tidak dimasukkan ke remaining → terhapus dari queue
                    }
                    Ok(res) => {
                        let status = res.status();
                        let body = res.text().await.unwrap_or_default();
                        eprintln!(
                            "[ATS Worker] ✗ entry {} gagal — server {status}: {body} (label: {})",
                            entry.id, entry.label
                        );
                        entry.attempt_count += 1;
                        remaining.push(entry);
                        consecutive_failures += 1;
                    }
                    Err(e) => {
                        eprintln!(
                            "[ATS Worker] ✗ entry {} gagal kirim: {e:?} (label: {})",
                            entry.id, entry.label
                        );
                        entry.attempt_count += 1;
                        remaining.push(entry);
                        consecutive_failures += 1;
                    }
                }
            }

            if any_success {
                if let Err(e) = queue.rewrite(&remaining).await {
                    eprintln!("[ATS Worker] gagal rewrite queue: {e}");
                }
                consecutive_failures = 0;
            }

            let wait_secs = if consecutive_failures == 0 {
                RETRY_BASE_SECS
            } else {
                let backoff = RETRY_BASE_SECS * (2u64.pow(consecutive_failures.min(6)));
                backoff.min(RETRY_MAX_SECS)
            };

            tokio::time::sleep(Duration::from_secs(wait_secs)).await;
        }
    });
}