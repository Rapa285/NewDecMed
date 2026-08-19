
use iota_sdk::{
    IotaClientBuilder,
    types::crypto::IotaKeyPair,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

use crate::audit_error::AuditError;

// ── Metadata struct ───────────────────────────────────────────────────────────


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IotaLogMetadata {
    pub version: String,
    pub log_sequence_number: u64,
    pub rotation_timestamp: DateTime<Utc>,
    pub ipfs_cid: String,
    pub file_hash: String,
    pub first_record_hash: String,
    pub final_record_hash: String,
    pub record_count: u64,
    pub prev_object_id: Option<String>,
}

// ── Hasil publish ─────────────────────────────────────────────────────────────

pub struct PublishResult {
    pub object_id: String,
    pub tx_digest: String,
}

// ── IotaLogClient ─────────────────────────────────────────────────────────────

pub struct IotaLogClient {
    node_url: String,
    key_pair: IotaKeyPair,
}

impl IotaLogClient {
    pub fn new(node_url: String, key_pair_bech32: String) -> Result<Self, AuditError> {
        let key_pair = IotaKeyPair::decode(&key_pair_bech32)
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal decode IOTA key pair: {e}")
            ))?;
        Ok(Self { node_url, key_pair })
    }

    pub async fn publish_metadata(
        &self,
        metadata: &IotaLogMetadata,
    ) -> Result<PublishResult, AuditError> {
        // ── 1. Serialize metadata ke JSON string ──────────────────────────
        let metadata_json = serde_json::to_string(metadata)
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal serialize metadata: {e}")
            ))?;

        // ── 2. Bangun IOTA client ─────────────────────────────────────────
        let iota_client = IotaClientBuilder::default()
            .build(&self.node_url)
            .await
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal connect ke IOTA node: {e}")
            ))?;

        let delete_lock_until: u32 = 4102444800;

        let result = notarization_client
            .create_locked()                          
            .with_state(metadata_json.as_bytes())     
            .with_description(format!(               
                "ATS Log #{} — {}",
                metadata.log_sequence_number,
                metadata.rotation_timestamp.format("%Y-%m-%dT%H:%M:%SZ")
            ))
            .with_delete_lock(TimeLock::UnlockAt(delete_lock_until))
            .finish()                                 
            .build_and_execute()                      
            .await
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal publish ke IOTA: {e}")
            ))?;

        // ── 5. Ekstrak Object ID dari response ────────────────────────────
        let object_id = result
            .notarization_object_id()
            .map(|id| id.to_string())
            .ok_or_else(|| AuditError::from(
                anyhow::anyhow!("IOTA response tidak mengandung object ID")
            ))?;

        let tx_digest = result
            .iota_response()
            .digest
            .to_string();

        println!(
            "[iota] metadata tersimpan. Object ID: {object_id} | TX: {tx_digest}"
        );

        Ok(PublishResult { object_id, tx_digest })
    }

    pub async fn verify_metadata(
        &self,
        object_id: &str,
        expected: &IotaLogMetadata,
    ) -> Result<bool, AuditError> {
        let iota_client = IotaClientBuilder::default()
            .build(&self.node_url)
            .await
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal connect ke IOTA node: {e}")
            ))?;

        let read_only = NotarizationClientReadOnly::new(iota_client)
            .await
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal buat read-only client: {e}")
            ))?;

        // Fetch notarized object tanpa perlu signing (read-only)
        let handle = read_only
            .notarization(object_id)
            .await
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal fetch object dari IOTA: {e}")
            ))?;

        let on_chain_state = handle
            .state()
            .await
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal baca state dari object: {e}")
            ))?;

        // Parse on-chain state kembali ke IotaLogMetadata
        let on_chain_metadata: IotaLogMetadata = serde_json::from_slice(
            on_chain_state.data()
        ).map_err(|e| AuditError::from(
            anyhow::anyhow!("gagal parse metadata dari IOTA: {e}")
        ))?;

        // Bandingkan field kritis
        let valid = on_chain_metadata.ipfs_cid == expected.ipfs_cid
            && on_chain_metadata.file_hash == expected.file_hash
            && on_chain_metadata.initial_public_key == expected.initial_public_key
            && on_chain_metadata.log_sequence_number == expected.log_sequence_number;

        Ok(valid)
    }

    /// Hitung SHA-256 hash dari file — untuk field `file_hash` di metadata
    pub async fn hash_file(file_path: &str) -> Result<String, AuditError> {
        use tokio::io::AsyncReadExt;

        let mut file = tokio::fs::File::open(file_path)
            .await
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal buka file untuk hash: {e}")
            ))?;

        let mut hasher = Sha256::new();
        let mut buf = [0u8; 8192];

        loop {
            let n = file.read(&mut buf).await
                .map_err(|e| AuditError::from(
                    anyhow::anyhow!("gagal baca file: {e}")
                ))?;
            if n == 0 { break; }
            hasher.update(&buf[..n]);
        }

        Ok(hex::encode(hasher.finalize()))
    }
}