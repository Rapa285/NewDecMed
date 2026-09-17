// src/ats/ats.rs

use iota_types::crypto::{EncodeDecodeBase64, IotaKeyPair, Signature as IotaSignature};
use shared_crypto::intent::{Intent, IntentMessage};
use reqwest::Client;

use super::constants::{ATS_ENDPOINT,ATS_QUEUE_DIR};
use super::queue::{new_queue_entry, spawn_retry_worker, AtsQueue};
use super::types::{AuditEvent, SignedAuditEvent};

pub struct ATSClient {
    req_client: Client,
    queue: AtsQueue,
}

impl ATSClient {
    pub fn new() -> Self {
        let queue = AtsQueue::new(ATS_QUEUE_DIR);
        spawn_retry_worker(queue.clone(), ATS_ENDPOINT);

        Self {
            req_client: Client::new(),
            queue,
        }
    }

    pub async fn send_event(
        &self,
        event: AuditEvent,
        iota_address: String,
        iota_key_pair: &IotaKeyPair,
        label: &str,
    ) -> Result<(), String> {
        // ── Langkah 1: Serialize dan sign event ──────────────────────────────
        let payload_string = serde_json::to_string(&event)
            .map_err(|e| format!("[ATS][{label}] gagal serialisasi event: {e:?}"))?;

        let intent_msg = IntentMessage::new(
            Intent::personal_message(),
            payload_string.as_bytes().to_vec(),
        );
        let signature = IotaSignature::new_secure(&intent_msg, iota_key_pair);

        let signed = SignedAuditEvent {
            payload: payload_string,
            signature: signature.encode_base64(),
            iota_address,
        };

        let signed_json = serde_json::to_string(&signed)
            .map_err(|e| format!("[ATS][{label}] gagal serialize SignedAuditEvent: {e:?}"))?;

        // ── Langkah 2: Simpan ke queue dulu (event aman di disk) ─────────────
        let entry = new_queue_entry(signed_json.clone(), label);
        let entry_id = entry.id.clone();

        self.queue.push(entry).await.map_err(|e| {
            format!(
                "[ATS][{label}] KRITIS: gagal simpan ke queue — event mungkin hilang: {e}"
            )
        })?;

        println!("[ATS][{label}] event disimpan ke queue (id: {entry_id})");

        // ── Langkah 3: Coba kirim langsung ───────────────────────────────────
        match self
            .req_client
            .post(ATS_ENDPOINT)
            .header("Content-Type", "application/json")
            .body(signed_json)
            .send()
            .await
        {
            Ok(res) if res.status().is_success() => {
                println!("[ATS][{label}] event {entry_id} langsung terkirim ✓");

                // Hapus dari queue karena sudah berhasil
                let mut entries = self.queue.read_all().await;
                entries.retain(|e| e.id != entry_id);
                if let Err(e) = self.queue.rewrite(&entries).await {
                    eprintln!("[ATS][{label}] gagal hapus entry {entry_id} dari queue: {e}");
                }
            }
            Ok(res) => {
                let status = res.status();
                let body = res.text().await.unwrap_or_default();
                eprintln!(
                    "[ATS][{label}] pengiriman langsung gagal (server {status}: {body}), \
                     event {entry_id} akan di-retry oleh background worker"
                );
            }
            Err(e) => {
                eprintln!(
                    "[ATS][{label}] pengiriman langsung gagal ({e:?}), \
                     event {entry_id} akan di-retry oleh background worker"
                );
            }
        }

        Ok(())
    }
}