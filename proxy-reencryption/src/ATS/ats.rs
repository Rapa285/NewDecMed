use ed25519_dalek::{Signature, Signer, Verifier, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use reqwest::{Client, StatusCode};
use serde::{Serialize, Deserialize};
use anyhow::{Context, Result, anyhow};

// --- Types ---
#[derive(Debug, Serialize, Deserialize)]
pub struct SignedAuditEvent {
    #[serde(flatten)] 
    pub event: AuditEvent,
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
        ttl_remaining: i64, // Dalam detik/milidetik
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
        operation_type: String, // Upload/Download
        data_size: u64, // Dalam bytes
    },

    #[serde(rename = "EV16")]
    IPFSVerification {
        cid: String,
        expected_hash: String,
        verification_result: bool,
    },

    #[serde(rename = "EV17")]
    LedgerQuery {
        metadata_type: String,
        object_id: String,
        query_requester_id: String,
    },

    #[serde(rename = "EV18")]
    CapabilityValidation {
        capability_checked: String,
        required_scope: String,
        actual_scope: String,
        validation_result: bool, // true jika valid, false jika tidak
    },
}


// --- Modul ATS ---
pub mod ats_module {

    const ATS_ENDPOINT: &str = "http://localhost:3000/api/events";
    const SOURCE_ID: &str = "decmed-tauri";

    pub struct ATSClient {
        private_key: SigningKey, 
        pub public_key: VerifyingKey, 
        req_client: Client,
    }

    impl ATSClient {
        pub fn new() -> Self {
            let (private_key, public_key) = Self::generate_keypair();
            
            Self {
                private_key,
                public_key,
                req_client: Client::new(),
            }
        }

        fn generate_keypair() -> (SigningKey, VerifyingKey) {
            let mut csprng = OsRng;
            let private_key = SigningKey::generate(&mut csprng);
            let public_key = private_key.verifying_key();
            
            (private_key, public_key)
        }

        pub async fn post_audit_event(
            &self,
            event: AuditEvent,
        ) -> Result<(), anyhow::Error> {
            
            let payload_bytes = serde_json::to_vec(&event)
                .context("Gagal melakukan serialisasi AuditEvent")?;

            // Menggunakan self.private_key untuk menandatangani
            let signature = self.private_key.sign(&payload_bytes);

            let signature_hex = hex::encode(signature.to_bytes());
            let pubkey_hex = hex::encode(self.public_key.as_bytes());

            let signed_payload = SignedAuditEvent {
                event,
                signature: signature_hex,
                public_key: pubkey_hex, // Mengirimkan public key agar server bisa memverifikasi
            };

            let res = self.req_client
                .post(ATS_ENDPOINT)
                .json(&signed_payload)
                .send()
                .await
                .context("Gagal mengirim request ke ATS Endpoint")?;

            let res_status = res.status();
            
            if !res_status.is_success() {
                let res_body = res.text().await.unwrap_or_default();
                return Err(anyhow!("ATS Server mengembalikan error {}: {}", res_status, res_body));
            }

            Ok(())
        }
    }
    
}