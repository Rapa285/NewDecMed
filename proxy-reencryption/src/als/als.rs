use base64::{engine::general_purpose::STANDARD, Engine as _};
use iota_types::crypto::{EncodeDecodeBase64, IotaKeyPair, Signature as IotaSignature};
use shared_crypto::intent::{Intent, IntentMessage};

use super::constants::{ALS_ENDPOINT, ALS_SERVER_PUBLIC_KEY};
use super::crypto::{aes_encrypt, ecies_encrypt_key};
use super::queue::{new_queue_entry, spawn_retry_worker, AlsQueue};
use super::types::{AuditEvent, EncryptedSignedEvent,AuditSourceComponent,Event};
use crate::types::AppState;

use chrono::Utc;


pub struct ALSClient;

impl ALSClient {
    /// Dipanggil sekali saat startup di main.rs
    pub fn start_retry_worker() {
        // println!("[ALS] Start Retry worker");
        spawn_retry_worker();
    }

    /// Entry point utama pengiriman event dari PRE server.
    /// Mengambil keypair dari AppState secara langsung.
    pub fn send_event_from_state(
        state: &AppState,
        event: Event,
        label: &'static str,
    ) {

        let audit_event = AuditEvent{
            source_component : AuditSourceComponent::ProxyReencryption,
            source_timestamp : Utc::now(),
            event : event,
        };

        // Decode proxy keypair dari state (plain encoded string, tidak perlu PIN)
        let key_pair_str = state.proxy_iota_key_pair.clone();
        let iota_address = state.proxy_iota_address.clone();

        tokio::spawn(async move {
            let iota_key_pair = match IotaKeyPair::decode(&key_pair_str) {
                Ok(kp) => kp,
                Err(e) => {
                    eprintln!("[ALS][{label}] gagal decode keypair: {e:?}");
                    return;
                }
            };

            Self::build_and_send(audit_event, iota_address, &iota_key_pair, label).await;
        });
    }

    /// Core logic: enkripsi + sign + queue + kirim
    async fn build_and_send(
        event: AuditEvent,
        iota_address: String,
        iota_key_pair: &IotaKeyPair,
        label: &'static str,
    ) {
        // ── Step 1: Serialize AuditEvent ──────────────────────────────────────
        let payload_bytes = match serde_json::to_vec(&event) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("[ALS][{label}] gagal serialize event: {e:?}");
                return;
            }
        };

        // ── Step 2: Enkripsi payload dengan AES-256-GCM ───────────────────────
        let encrypted = match aes_encrypt(&payload_bytes) {
            Ok(enc) => enc,
            Err(e) => {
                eprintln!("[ALS][{label}] gagal enkripsi payload: {e}");
                return;
            }
        };

        // ── Step 3: Enkripsi AES key dengan public key ALS (ECIES) ───────────
        let enc_aes_key = match ecies_encrypt_key(&encrypted.key, ALS_SERVER_PUBLIC_KEY) {
            Ok(k) => k,
            Err(e) => {
                eprintln!("[ALS][{label}] gagal enkripsi AES key: {e}");
                return;
            }
        };

        // ── Step 4: Sign ciphertext dengan IotaKeyPair (Encrypt-then-Sign) ───
        //    Yang ditandatangani adalah ciphertext, bukan plaintext.
        //    Ini membuktikan pengirim yang membuat ciphertext ini.
        let intent_msg = IntentMessage::new(
            Intent::personal_message(),
            encrypted.ciphertext.clone(),
        );
        let signature = IotaSignature::new_secure(&intent_msg, iota_key_pair);

        // ── Step 5: Bangun EncryptedSignedEvent ───────────────────────────────
        let signed_event = EncryptedSignedEvent {
            enc_aes_key,
            ciphertext: STANDARD.encode(&encrypted.ciphertext),
            nonce: STANDARD.encode(&encrypted.nonce),
            signature: signature.encode_base64(),
            iota_address: iota_address.clone(),
        };

        let signed_json = match serde_json::to_string(&signed_event) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("[ALS][{label}] gagal serialize EncryptedSignedEvent: {e:?}");
                return;
            }
        };

        // ── Step 6: Simpan ke queue dulu (durability) ─────────────────────────
        let entry = new_queue_entry(signed_json.clone(), label);
        let entry_id = entry.id.clone();

        if let Err(e) = AlsQueue::push(entry).await {
            eprintln!(
                "[ALS][{label}] KRITIS: gagal simpan ke queue \
                 — event mungkin hilang: {e}"
            );
            return;
        }

        println!("[ALS][{label}] event disimpan ke queue (id: {entry_id})");

        // ── Step 7: Coba kirim langsung ───────────────────────────────────────
        let client = reqwest::Client::new();

        match client
            .post(ALS_ENDPOINT)
            .header("Content-Type", "application/json")
            .body(signed_json)
            .send()
            .await
        {
            Ok(res) if res.status().is_success() => {
                println!("[ALS][{label}] event {entry_id} terkirim langsung ✓");

                // Hapus dari queue karena sudah berhasil
                let mut entries = AlsQueue::read_all().await;
                entries.retain(|e| e.id != entry_id);
                if let Err(e) = AlsQueue::rewrite(&entries).await {
                    eprintln!("[ALS][{label}] gagal hapus {entry_id} dari queue: {e}");
                }
            }
            Ok(res) => {
                eprintln!(
                    "[ALS][{label}] kirim langsung gagal (server {}), \
                     entry {entry_id} akan di-retry",
                    res.status()
                );
            }
            Err(e) => {
                eprintln!(
                    "[ALS][{label}] kirim langsung gagal ({e:?}), \
                     entry {entry_id} akan di-retry"
                );
            }
        }
    }
}