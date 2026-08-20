use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result, anyhow};

// ── Tipe data publik ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedAuditEvent {
    pub payload: String,
    pub signature: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub source_component: String,
    pub actor_id: String,
    pub target_object: String,
    pub outcome: AuditOutcome,
    pub action_type: String,

    #[serde(flatten)]
    pub details: AuditEventDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditOutcome {
    Success,
    Failure,
    Denied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum AuditEventDetails {
    #[serde(rename = "EV6")]
    PRERequest {
        endpoint_called: String,
        request_id: String,
        caller_component: String,
        channel_encryption: String,
    },
    #[serde(rename = "EV7")]
    Reencryption {
        reencryption_operation_id: String,
        capability_id: String,
        target_ciphertext: String,
        kfrag_identifier: String,
    },
    #[serde(rename = "EV13")]
    RedisWrite {
        redis_key_type: String,
        operation_type: String,
        ttl_remaining: i64,
    },
    #[serde(rename = "EV14")]
    RedisRead {
        redis_key_type: String,
        request_origin: String,
        ttl_remaining: i64,
    },
    #[serde(rename = "EV15")]
    IPFSOperation {
        cid: String,
        operation_type: String,
        data_size: u64,
    },
}

// ── ATSClient ─────────────────────────────────────────────────────────────────

pub struct ATSClient {
    signing_key: SigningKey,
    pub public_key: VerifyingKey,
    req_client: Client,
    endpoint: String,
}

impl ATSClient {
    const DEFAULT_ENDPOINT: &'static str = "http://localhost:3000/api/events";

    pub fn new() -> Self {
        Self::with_endpoint(Self::DEFAULT_ENDPOINT)
    }

    pub fn with_endpoint(endpoint: &str) -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key();
        Self {
            signing_key,
            public_key,
            req_client: Client::new(),
            endpoint: endpoint.to_string(),
        }
    }

    /// Kirim event secara sinkron (await). Gunakan ini jika Anda butuh menangani error.
    pub async fn send_event(&self, event: AuditEvent) -> Result<()> {

        let payload_string = serde_json::to_string(&event)?;
        let payload_bytes = payload_string.as_bytes();

        let signature: Signature = self.signing_key.sign(&payload_bytes);

        let signed = SignedAuditEvent {
            payload: payload_string,
            signature: hex::encode(signature.to_bytes()),
            public_key: hex::encode(self.public_key.as_bytes()),
        };

        let res = self.req_client
            .post(&self.endpoint)
            .json(&signed)
            .send()
            .await
            .context("gagal mengirim request ke ATS")?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(anyhow!("ATS error {}: {}", status, body));
        }

        Ok(())
    }

    /// Fire-and-forget: spawn task Tokio terpisah.
    /// Error hanya dicatat ke stderr, tidak memblokir caller.
    pub fn send_event_nonblocking(&self, event: AuditEvent, label: &'static str) {
        // Salin semua data yang dibutuhkan task agar 'static
        let signing_key_bytes = self.signing_key.to_bytes();
        let pubkey_bytes = self.public_key.to_bytes();
        let client = self.req_client.clone();
        let endpoint = self.endpoint.clone();

        tokio::spawn(async move {
            let signing_key = SigningKey::from_bytes(&signing_key_bytes);
            let worker = ATSWorker { signing_key, pubkey_bytes, client, endpoint };
            if let Err(e) = worker.send(event).await {
                eprintln!("[ATS][{}] gagal kirim event: {e:?}", label);
            }
        });
    }
}

// Helper internal — dimiliki penuh oleh spawned task
struct ATSWorker {
    signing_key: SigningKey,
    pubkey_bytes: [u8; 32],
    client: Client,
    endpoint: String,
}

impl ATSWorker {
    async fn send(&self, event: AuditEvent) -> Result<()> {
        let payload_string = serde_json::to_string(&event)?;

        let payload_bytes = payload_string.as_bytes();
        let signature: Signature = self.signing_key.sign(&payload_bytes);

        let signed = SignedAuditEvent {
            payload: payload_string,
            signature: hex::encode(signature.to_bytes()),
            public_key: hex::encode(self.pubkey_bytes),
        };

        // ─── TAMBAHKAN PRINT DI SINI ─────────────────────────────────────────
        // Cetak data dalam format JSON yang rapi (Pretty Print)
        if let Ok(json_debug) = serde_json::to_string_pretty(&signed) {
            println!("[ATS Worker] Akan mengirim event ke endpoint '{}':\n{}", self.endpoint, json_debug);
        } else {
            // Fallback jika pretty print gagal (membutuhkan trait Debug pada SignedAuditEvent)
            println!("[ATS Worker] Akan mengirim event: {:?}", signed);
        }
        // ─────────────────────────────────────────────────────────────────────

        let res = self.client.post(&self.endpoint).json(&signed).send().await?;
        if !res.status().is_success() {
            let s = res.status();
            let b = res.text().await.unwrap_or_default();
            return Err(anyhow!("ATS {s}: {b}"));
        }
        Ok(())
    }
}