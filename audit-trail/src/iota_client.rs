use chrono::{DateTime, Utc};
use iota_json_rpc_types::{
    IotaObjectData, IotaObjectDataFilter, IotaObjectDataOptions, IotaObjectResponseQuery,
};
use iota_types::{
    base_types::ObjectID,
    crypto::{IotaKeyPair, Signature},
    transaction::{CallArg, Transaction},
    Identifier,
};
use move_core_types::{account_address::AccountAddress, language_storage::StructTag};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shared_crypto::intent::{Intent, IntentMessage};
use std::str::FromStr;
use tokio::fs;

use crate::audit_error::AuditError;
use crate::constants::IOTA_KEY_PAIR;
use crate::iota_utils::IotaUtils;

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
    pub prev_tx_digest: Option<String>,
}

// ── Hasil publish ─────────────────────────────────────────────────────────────

pub struct PublishResult {
    pub object_id: ObjectID,
    pub tx_digest: String,
}

// ── LogRecord on-chain (hasil fetch object) ───────────────────────────────────

/// Representasi `LogRecord` yang di-fetch dari IOTA,
/// sebelum di-parse ke `IotaLogMetadata`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecordOnChain {
    pub object_id: ObjectID,
    pub metadata: IotaLogMetadata,
}

// ── IotaLogClient ─────────────────────────────────────────────────────────────

pub struct IotaLogClient {
    key_pair: IotaKeyPair,
    package_id: ObjectID,
}

impl IotaLogClient {
    pub fn new(package_id_hex: &str) -> Result<Self, AuditError> {
        let key_pair = IotaKeyPair::decode(IOTA_KEY_PAIR)
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal decode IOTA key pair: {e}")
            ))?;

        let package_id = ObjectID::from_hex_literal(package_id_hex)
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("Invalid Package ID: {e}")
            ))?;

        Ok(Self { key_pair, package_id })
    }

    // ── publish_metadata ──────────────────────────────────────────────────────

    /// Memanggil `create_log(json_data, ctx)` di Move untuk membuat frozen `LogRecord`.
    /// Mengembalikan `object_id` dari LogRecord yang baru dibuat dan `tx_digest`.
    pub async fn publish_metadata(
        &self,
        metadata: &IotaLogMetadata,
    ) -> Result<PublishResult, AuditError> {
        // 1. Serialize metadata ke JSON string
        let metadata_json = serde_json::to_string(metadata)
            .map_err(|e| anyhow::anyhow!("gagal serialize metadata: {e}"))?;

        // 2. Bangun IOTA client via IotaUtils
        let iota_client = IotaUtils::get_iota_client().await?;
        let sender: iota_types::base_types::IotaAddress = (&self.key_pair.public()).into();

        // 3. Susun call argument — json_data bertipe String di Move,
        //    di-encode sebagai BCS byte vector (Move String = BCS Vec<u8> UTF-8)
        let call_args: Vec<CallArg> = vec![
            CallArg::Pure(
                bcs::to_bytes(&metadata_json)
                    .map_err(|e| anyhow::anyhow!("gagal encode argument BCS: {e}"))?,
            ),
        ];

        // 4. Build ProgrammableTransaction via IotaUtils::construct_pt
        //    Module: audit_log | Fungsi: create_log
        let module = Identifier::from_str("audit_log")
            .map_err(|e| anyhow::anyhow!("nama module tidak valid: {e}"))?;

        let pt = IotaUtils::construct_pt(
            "create_log".to_string(),
            self.package_id,
            module,
            vec![],     // tidak ada type argument
            call_args,
        )?;

        // 5. Reserve gas via IotaUtils (gas station / sponsored tx)
        let (sponsor_address, reservation_id, gas_coins) =
            IotaUtils::reserve_gas(10_000_000, 60).await?;

        // 6. Construct sponsored TransactionData
        let ref_gas_price = IotaUtils::get_ref_gas_price(&iota_client).await?;

        let tx_data = IotaUtils::construct_sponsored_tx_data(
            sender,
            gas_coins,
            pt,
            10_000_000,
            ref_gas_price,
            sponsor_address,
        );

        // 7. Sign dan execute
        let intent_msg = IntentMessage::new(Intent::iota_transaction(), &tx_data);
        let signature = Signature::new_secure(&intent_msg, &self.key_pair);
        let tx = Transaction::from_data(tx_data, vec![signature]);

        let exec_response = IotaUtils::execute_tx(tx.into_inner(), reservation_id).await?;
        IotaUtils::handle_error_execute_tx(exec_response.clone())?;

        // 8. Ekstrak tx_digest dan object_id LogRecord yang baru di-freeze
        let effects = exec_response
            .effects
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("effects tidak tersedia di response"))?;

        let tx_digest = effects.transaction_digest().to_string();

        // LogRecord yang di-freeze muncul di `created` lalu masuk `unwrapped_then_deleted`
        // Pada IOTA Rebased, frozen object muncul di effects.created
        let object_id = effects
            .created()
            .first()
            .map(|obj_ref| obj_ref.reference.object_id)
            .ok_or_else(|| anyhow::anyhow!("object_id LogRecord tidak ditemukan di effects"))?;

        println!(
            "[iota] LogRecord dibuat. Object ID: {object_id} | TX Digest: {tx_digest}"
        );

        Ok(PublishResult { object_id, tx_digest })
    }

    pub async fn get_log_records(
        &self,
        limit: Option<usize>,
    ) -> Result<Vec<LogRecordOnChain>, AuditError> {
        let iota_client = IotaUtils::get_iota_client().await?;

        // Filter berdasarkan StructType `<package_id>::audit_log::LogRecord`
        let struct_tag = StructTag {
            address: AccountAddress::from(self.package_id),
            module: Identifier::from_str("audit_log")
                .map_err(|e| anyhow::anyhow!("module identifier tidak valid: {e}"))?,
            name: Identifier::from_str("LogRecord")
                .map_err(|e| anyhow::anyhow!("struct name identifier tidak valid: {e}"))?,
            type_params: vec![],
        };

        let query = IotaObjectResponseQuery {
            filter: Some(IotaObjectDataFilter::StructType(struct_tag)),
            options: Some(IotaObjectDataOptions {
                show_content: true,     // agar json_data bisa dibaca
                show_type: true,
                show_owner: true,
                ..Default::default()
            }),
        };

        let mut results: Vec<LogRecordOnChain> = Vec::new();
        let mut cursor: Option<iota_types::base_types::ObjectID> = None;
        // Ambil per halaman 50 object; sesuaikan jika perlu
        let page_size: usize = 50;

        'pagination: loop {
            // Hitung sisa yang masih dibutuhkan jika limit ada
            let fetch_size = match limit {
                Some(lim) => page_size.min(lim.saturating_sub(results.len())),
                None => page_size,
            };

            if fetch_size == 0 {
                break;
            }

            let page = iota_client
                .read_api()
                .get_owned_objects(
                    // LogRecord adalah frozen object; owner-nya adalah @0x0 (Immutable)
                    // Gunakan query_objects untuk object tanpa owner spesifik
                    iota_types::base_types::IotaAddress::ZERO,
                    query.clone(),
                    cursor,
                    Some(fetch_size),
                )
                .await
                .map_err(|e| anyhow::anyhow!("gagal query LogRecord dari IOTA: {e}"))?;

            for response in &page.data {
                let object_data: &IotaObjectData = response
                    .data
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("object data kosong di response"))?;

                let record = Self::parse_log_record(object_data)?;
                results.push(record);

                if let Some(lim) = limit {
                    if results.len() >= lim {
                        break 'pagination;
                    }
                }
            }

            if page.has_next_page {
                cursor = page.next_cursor;
            } else {
                break;
            }
        }

        Ok(results)
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    /// Parse `IotaObjectData` → `LogRecordOnChain`.
    /// Mengekstrak field `json_data` dari content object lalu deserialize ke `IotaLogMetadata`.
    fn parse_log_record(object_data: &IotaObjectData) -> Result<LogRecordOnChain, AuditError> {
        let object_id = object_data.object_id;

        // Content object tersedia karena show_content: true di query options
        let content = object_data
            .content
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("content object {object_id} kosong"))?;

        // IotaObjectData content adalah IotaParsedData::MoveObject
        let move_object = match content {
            iota_json_rpc_types::IotaParsedData::MoveObject(obj) => obj,
            _ => return Err(anyhow::anyhow!("object {object_id} bukan MoveObject").into()),
        };

        // Fields direpresentasikan sebagai serde_json::Value
        let json_data_str = move_object
            .fields
            .get("json_data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!("field json_data tidak ditemukan pada object {object_id}")
            })?;

        let metadata: IotaLogMetadata = serde_json::from_str(json_data_str)
            .map_err(|e| {
                anyhow::anyhow!("gagal parse IotaLogMetadata dari object {object_id}: {e}")
            })?;

        Ok(LogRecordOnChain { object_id, metadata })
    }

    // ── hash_file ─────────────────────────────────────────────────────────────

    /// Hash file menggunakan SHA-256.
    pub async fn hash_file(file_path: &str) -> Result<String, AuditError> {
        let bytes = fs::read(file_path)
            .await
            .map_err(|e| anyhow::anyhow!("gagal baca file untuk hashing: {e}"))?;

        let mut hasher = Sha256::new();
        hasher.update(&bytes);

        Ok(hex::encode(hasher.finalize()))
    }
}