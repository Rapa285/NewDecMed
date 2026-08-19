use iota_sdk::{
    IotaClientBuilder,
    types::{
        crypto::{IotaKeyPair, Signature},
        base_types::ObjectID,
    },
    rpc_types::IotaTransactionBlockResponseOptions,
};
use fastcrypto::traits::KeyPair;
use shared_crypto::intent::{Intent, IntentMessage};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::str::FromStr;

use crate::audit_error::AuditError;
use crate::constants::IOTA_KEY_PAIR;

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
    pub prev_tx_digest: Option<String>, // Ubah dari object_id ke tx_digest
}

// ── Hasil publish ─────────────────────────────────────────────────────────────

pub struct PublishResult {
    pub tx_digest: String, // Di Rebased (Event), referensi utamanya adalah TX Digest
}

// ── IotaLogClient ─────────────────────────────────────────────────────────────

pub struct IotaLogClient {
    node_url: String,
    key_pair: IotaKeyPair,
    package_id: ObjectID, // ID dari Move Package yang sudah di-deploy
}

impl IotaLogClient {
    pub fn new(node_url: String, package_id_hex: &str) -> Result<Self, AuditError> {
        let key_pair = IotaKeyPair::decode(IOTA_KEY_PAIR)
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal decode IOTA key pair: {e}")
            ))?;
            
        let package_id = ObjectID::from_hex_literal(package_id_hex)
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("Invalid Package ID: {e}")
            ))?;

        Ok(Self { node_url, key_pair, package_id })
    }

    pub async fn publish_metadata(
        &self,
        metadata: &IotaLogMetadata,
    ) -> Result<PublishResult, AuditError> {
        // 1. Serialize metadata ke JSON string
        let metadata_json = serde_json::to_string(metadata)
            .map_err(|e| anyhow::anyhow!("gagal serialize metadata: {e}"))?;

        // 2. Bangun IOTA Rebased client
        let client = IotaClientBuilder::default()
            .build(&self.node_url)
            .await
            .map_err(|e| anyhow::anyhow!("gagal connect ke IOTA node: {e}"))?;

        let sender = (&self.key_pair.public()).into();

        // 3. Bangun Transaction (Memanggil fungsi Move untuk memancarkan Event)
        let tx_data = client
            .transaction_builder()
            .move_call(
                sender,
                self.package_id,
                "audit_log",      // Nama module di Move
                "publish_log",    // Nama fungsi di Move
                vec![],           // Type arguments (kosong)
                vec![             // Arguments (JSON String kita)
                    iota_sdk::json::IotaJsonValue::from_str(&format!("\"{}\"", metadata_json))
                        .map_err(|e| anyhow::anyhow!("Gagal parse argument JSON: {e}"))?
                ],
                None,             // Gas object (None = biarkan SDK yang memilih coin otomatis)
                10_000_000,       // Gas budget (sesuaikan)
                None,
            )
            .await
            .map_err(|e| anyhow::anyhow!("Gagal build tx: {e}"))?;

        // 4. Sign Transaksi
        let intent_message = IntentMessage::new(Intent::iota_transaction(), &tx_data);
        let signature = Signature::new_secure(&intent_message, &self.key_pair);

        // 5. Eksekusi Transaksi
        let response = client
            .quorum_driver_api()
            .execute_transaction_block(
                iota_sdk::types::transaction::Transaction::from_data(tx_data, vec![signature]),
                IotaTransactionBlockResponseOptions::new(),
                None,
            )
            .await
            .map_err(|e| anyhow::anyhow!("gagal publish ke IOTA Rebased: {e}"))?;

        let tx_digest = response.digest.to_string();
        println!("[iota] metadata tersimpan sebagai Event. TX Digest: {tx_digest}");

        Ok(PublishResult { tx_digest })
    }

    pub async fn verify_metadata(
        &self,
        tx_digest: &str,
        expected: &IotaLogMetadata,
    ) -> Result<bool, AuditError> {
        let client = IotaClientBuilder::default()
            .build(&self.node_url)
            .await
            .map_err(|e| anyhow::anyhow!("gagal connect ke IOTA node: {e}"))?;

        let digest = iota_sdk::types::digests::TransactionDigest::from_str(tx_digest)
            .map_err(|e| anyhow::anyhow!("Format TX Digest tidak valid: {e}"))?;

        // 1. Fetch Events dari transaksi tersebut
        let events = client
            .event_api()
            .get_events(digest)
            .await
            .map_err(|e| anyhow::anyhow!("gagal fetch events dari IOTA: {e}"))?;

        // 2. Ambil event pertama (karena kita hanya me-emit satu log per TX)
        let log_event = events.data.first().ok_or_else(|| {
            anyhow::anyhow!("Tidak ada event ditemukan di TX ini")
        })?;

        // 3. Ekstrak data JSON dari dalam field event
        // Pastikan field di Move struct bernama `json_data`
        let on_chain_json = log_event.parsed_json.get("json_data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Field json_data tidak ditemukan pada Event"))?;

        let on_chain_metadata: IotaLogMetadata = serde_json::from_str(on_chain_json)
            .map_err(|e| anyhow::anyhow!("gagal parse metadata dari on-chain Event: {e}"))?;

        // 4. Bandingkan field kritis
        let valid = on_chain_metadata.ipfs_cid == expected.ipfs_cid
            && on_chain_metadata.file_hash == expected.file_hash
            && on_chain_metadata.log_sequence_number == expected.log_sequence_number;

        Ok(valid)
    }

    // Fungsi hash_file tetap sama persis seperti sebelumnya
    // ...
}