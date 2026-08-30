use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use reqwest::Client;
use super::types::{AuditEvent,SignedAuditEvent};
use super::constants::ATS_ENDPOINT;


// ── ATSClient ─────────────────────────────────────────────────────────────────

pub struct ATSClient {
    signing_key: SigningKey,
    pub public_key: VerifyingKey,
    req_client: Client,
}

impl ATSClient {

    pub fn new() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key();
        Self {
            signing_key,
            public_key,
            req_client: Client::new(),
        }
    }

    pub fn send_event(&self, event: AuditEvent, label: &'static str) {

        let signing_key_bytes = self.signing_key.to_bytes();
        let pubkey_bytes = self.public_key.to_bytes();
        let client = self.req_client.clone();
        let endpoint = ATS_ENDPOINT;

        tokio::spawn(async move {
            let signing_key = SigningKey::from_bytes(&signing_key_bytes);
            
            // Logika pengiriman langsung di sini
            let payload_string = match serde_json::to_string(&event) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("[ATS][{}] gagal serialisasi event: {e:?}", label);
                    return;
                }
            };

            let payload_bytes = payload_string.as_bytes();
            let signature: Signature = signing_key.sign(payload_bytes);

            let signed = SignedAuditEvent {
                payload: payload_string,
                signature: hex::encode(signature.to_bytes()),
                public_key: hex::encode(pubkey_bytes),
            };

            if let Ok(json_debug) = serde_json::to_string_pretty(&signed) {
                println!("[ATS Worker] Akan mengirim event ke endpoint '{}':\n{}", endpoint, json_debug);
            }

            match client.post(endpoint).json(&signed).send().await {
                Ok(res) if !res.status().is_success() => {
                    let s = res.status();
                    let b = res.text().await.unwrap_or_default();
                    eprintln!("[ATS][{}] ATS error {s}: {b}", label);
                }
                Err(e) => {
                    eprintln!("[ATS][{}] gagal kirim request: {e:?}", label);
                }
                _ => {}
            }
        });
    }
}