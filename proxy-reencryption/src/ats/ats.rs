// src/ats/ats.rs

use iota_types::crypto::{EncodeDecodeBase64, IotaKeyPair, Signature as IotaSignature};
use shared_crypto::intent::{Intent, IntentMessage};
use aes_gcm::{aead::Aead, AeadCore, Aes256Gcm, KeyInit, Nonce};
use rand::rngs::OsRng;

use super::constants::ATS_ENDPOINT;
use super::queue::{new_queue_entry, spawn_retry_worker, AtsQueue};
use super::types::{AuditEvent, SignedAuditEvent};
use crate::utils;
use crate::types::AppState;

pub struct ATSClient;

impl ATSClient {
    /// Dipanggil sekali saat startup aplikasi
    pub fn start_retry_worker() {
        spawn_retry_worker(ATS_ENDPOINT);
    }

    pub fn send_event(
        event: AuditEvent,
        iota_address: String,
        iota_key_pair: &IotaKeyPair,
        label: &'static str,
    ) {
        // ── Langkah 1: Serialize dan sign event ──────────────────────────────
        let payload_string = match serde_json::to_string(&event) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[ATS][{label}] gagal serialisasi event: {e:?}");
                return;
            }
        };

        // 1. Generate AES key + enkripsi payload
        let aes_key = Aes256Gcm::generate_key(OsRng);
        let cipher = Aes256Gcm::new(&aes_key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher.encrypt(&nonce, serde_json::to_vec(&event)).unwrap();

        // 2. Enkripsi AES key dengan public key ATS (ECIES/ECDH)
        let enc_key = ecies_encrypt(&ATS_SERVER_PUBKEY, &aes_key);

        let intent_msg = IntentMessage::new(
            Intent::personal_message(),
            payload_string.as_bytes().to_vec(),
        );

        let signature = IotaSignature::new_secure(&intent_msg, iota_key_pair);

        let signed = SignedAuditEvent {
            enc_key: enc_key.encode_base64(),
            ciphertext: ciphertext.encode_base64(),
            nonce: nonce.encode_base64(),
            signature: signature.encode_base64(),
            public_key: iota_key_pair.public().encode_hex(),
            iota_address,
        };

        let signed_json = match serde_json::to_string(&signed) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("[ATS][{label}] gagal serialize SignedAuditEvent: {e:?}");
                return;
            }
        };

        // Pindah ke tokio::spawn karena operasi async (queue + HTTP)
        // signed_json dan label di-move ke dalam spawn
        tokio::spawn(async move {
            // ── Langkah 2: Simpan ke queue dulu (event aman di disk) ─────────
            let entry = new_queue_entry(signed_json.clone(), label);
            let entry_id = entry.id.clone();

            if let Err(e) = AtsQueue::push(entry).await {
                eprintln!(
                    "[ATS][{label}] KRITIS: gagal simpan ke queue \
                     — event mungkin hilang: {e}"
                );
                return;
            }

            println!("[ATS][{label}] event disimpan ke queue (id: {entry_id})");

            // ── Langkah 3: Coba kirim langsung ───────────────────────────────
            let client = reqwest::Client::new();

            match client
                .post(ATS_ENDPOINT)
                .header("Content-Type", "application/json")
                .body(signed_json)
                .send()
                .await
            {
                Ok(res) if res.status().is_success() => {
                    println!("[ATS][{label}] event {entry_id} langsung terkirim ✓");

                    // Hapus dari queue karena sudah berhasil
                    let mut entries = AtsQueue::read_all().await;
                    entries.retain(|e| e.id != entry_id);
                    if let Err(e) = AtsQueue::rewrite(&entries).await {
                        eprintln!(
                            "[ATS][{label}] gagal hapus entry {entry_id} dari queue: {e}"
                        );
                    }
                }
                Ok(res) => {
                    let status = res.status();
                    let body = res.text().await.unwrap_or_default();
                    eprintln!(
                        "[ATS][{label}] pengiriman langsung gagal \
                         (server {status}: {body}), \
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
        });
    }

    /// Untuk konteks di mana keypair perlu didapat dari keys_entry
    pub fn send_event_from_state(
        state: &AppState,
        event: AuditEvent,
        label: &'static str,
    ) {
        // proxy_iota_key_pair sudah tersimpan sebagai String encoded
        let iota_key_pair = match IotaKeyPair::decode(&state.proxy_iota_key_pair) {
            Ok(kp) => kp,
            Err(e) => {
                eprintln!("[ATS][{label}] gagal decode proxy keypair: {e:?}");
                return;
            }
        };

        Self::send_event(
            event,
            state.proxy_iota_address.clone(),
            &iota_key_pair,
            label,
        );
    }
}

/**
* output:
* key: 32 bytes
* nonce: 12 bytes
* return: (ciphertext, key, nonce)
*/
pub fn aes_encrypt(data: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), HospitalError> {
    let key = Aes256Gcm::generate_key(aes_gcm::aead::OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut aes_gcm::aead::OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, data)
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;

    Ok((ciphertext, key.to_vec(), nonce.to_vec()))
}

pub fn aes_encrypt_custom_key(
    key: &[u8],
    data: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), HospitalError> {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let nonce = Aes256Gcm::generate_nonce(&mut aes_gcm::aead::OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, data)
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;

    Ok((ciphertext, nonce.to_vec()))
}

pub fn aes_decrypt(ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, HospitalError> {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let nonce = Nonce::from_slice(nonce);

    let original = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;

    Ok(original)
}