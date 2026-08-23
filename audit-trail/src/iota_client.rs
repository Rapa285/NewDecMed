use chrono::{DateTime, Utc};
use iota_json_rpc_types::{
    IotaObjectData, IotaObjectDataFilter, IotaObjectDataOptions, IotaObjectResponseQuery,
    IotaTransactionBlockEffectsAPI, // ← wajib di-import agar .transaction_digest() dan .created() tersedia
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecordOnChain {
    pub object_id: ObjectID,
    pub metadata: IotaLogMetadata,
}

/// Single page of `LogRecordOnChain`s, returned by `list_log_records`.
/// Mirrors the pagination shape the `audit-trail-client` (Tauri) app expects
/// from `GET /api/logs`.
pub struct LogRecordsPage {
    pub records: Vec<LogRecordOnChain>,
    pub next_cursor: Option<ObjectID>,
    pub has_next_page: bool,
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

    fn log_record_struct_tag(&self) -> Result<StructTag, AuditError> {
        Ok(StructTag {
            address: AccountAddress::from(self.package_id),
            module: Identifier::new("audit_log")
                .map_err(|e| anyhow::anyhow!("module identifier tidak valid: {e}"))?,
            name: Identifier::new("LogRecord")
                .map_err(|e| anyhow::anyhow!("struct name identifier tidak valid: {e}"))?,
            type_params: vec![],
        })
    }

    // ── publish_metadata ──────────────────────────────────────────────────────

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

        // 3. Encode argument BCS
        let call_args: Vec<CallArg> = vec![
            CallArg::Pure(
                bcs::to_bytes(&metadata_json)
                    .map_err(|e| anyhow::anyhow!("gagal encode argument BCS: {e}"))?,
            ),
        ];

        // 4. Build ProgrammableTransaction
        let module = Identifier::new("audit_log")
            .map_err(|e| anyhow::anyhow!("nama module tidak valid: {e}"))?;

        let pt = IotaUtils::construct_pt(
            "create_log".to_string(),
            self.package_id,
            module,
            vec![],
            call_args,
        )?;

        // 5. Reserve gas
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

        // 7. Sign
        let intent_msg = IntentMessage::new(Intent::iota_transaction(), &tx_data);
        let signature = Signature::new_secure(&intent_msg, &self.key_pair);

        // Transaction::from_data mengembalikan Transaction (bukan Envelope langsung)
        let tx: Transaction = Transaction::from_data(tx_data, vec![signature]);

        // 8. Execute — kirim sebagai Transaction (sesuai signature IotaUtils::execute_tx)
        let exec_response = IotaUtils::execute_tx(tx, reservation_id).await?;

        // Clone tidak lagi diperlukan karena handle_error_execute_tx mengonsumsi exec_response
        // Kita perlu effects dulu sebelum consume, jadi ekstrak dulu
        let effects_opt = exec_response.effects.clone();
        let error_opt = exec_response.error.clone();

        // Periksa error manual (hindari double-consume)
        if let Some(err) = error_opt {
            return Err(anyhow::anyhow!("execute_tx error: {err}").into());
        }

        let effects = effects_opt
            .ok_or_else(|| anyhow::anyhow!("effects tidak tersedia di response"))?;

        // IotaTransactionBlockEffectsAPI sudah di-import, method tersedia
        let tx_digest = effects.transaction_digest().to_string();

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
        let struct_tag = self.log_record_struct_tag()?;

        let query = IotaObjectResponseQuery {
            filter: Some(IotaObjectDataFilter::StructType(struct_tag)),
            options: Some(IotaObjectDataOptions {
                show_content: true,
                show_type: true,
                show_owner: true,
                ..Default::default()
            }),
        };

        let mut results: Vec<LogRecordOnChain> = Vec::new();
        let mut cursor: Option<iota_types::base_types::ObjectID> = None;
        let page_size: usize = 50;

        'pagination: loop {
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

    /// Fetch a single page of `LogRecord` objects, honoring an
    /// external opaque cursor. Used by `GET /api/logs` so the client
    /// can page through results instead of the server always
    /// re-walking from the start (as `get_log_records` does).
    pub async fn list_log_records(
        &self,
        cursor: Option<ObjectID>,
        limit: usize,
    ) -> Result<LogRecordsPage, AuditError> {
        let iota_client = IotaUtils::get_iota_client().await?;
        let struct_tag = self.log_record_struct_tag()?;

        let query = IotaObjectResponseQuery {
            filter: Some(IotaObjectDataFilter::StructType(struct_tag)),
            options: Some(IotaObjectDataOptions {
                show_content: true,
                show_type: true,
                show_owner: true,
                ..Default::default()
            }),
        };

        let page = iota_client
            .read_api()
            .get_owned_objects(
                iota_types::base_types::IotaAddress::ZERO,
                query,
                cursor,
                Some(limit),
            )
            .await
            .map_err(|e| anyhow::anyhow!("gagal query LogRecord dari IOTA: {e}"))?;

        let mut records = Vec::with_capacity(page.data.len());
        for response in &page.data {
            let object_data: &IotaObjectData = response
                .data
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("object data kosong di response"))?;

            records.push(Self::parse_log_record(object_data)?);
        }

        Ok(LogRecordsPage {
            records,
            next_cursor: page.next_cursor,
            has_next_page: page.has_next_page,
        })
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    fn parse_log_record(object_data: &IotaObjectData) -> Result<LogRecordOnChain, AuditError> {
        let object_id = object_data.object_id;

        let content = object_data
            .content
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("content object {object_id} kosong"))?;

        let move_object = match content {
            iota_json_rpc_types::IotaParsedData::MoveObject(obj) => obj,
            _ => return Err(anyhow::anyhow!("object {object_id} bukan MoveObject").into()),
        };

        // IotaMoveStruct adalah enum; gunakan pattern matching untuk akses field
        // Biasanya IotaMoveObject.fields bertipe IotaMoveStruct::WithFields(BTreeMap)
        let json_data_str = match &move_object.fields {
            iota_json_rpc_types::IotaMoveStruct::WithFields(fields) => {
                fields
                    .get("json_data")
                    .and_then(|v| {
                        // IotaMoveValue::String
                        if let iota_json_rpc_types::IotaMoveValue::String(s) = v {
                            Some(s.as_str())
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| {
                        anyhow::anyhow!("field json_data tidak ditemukan pada object {object_id}")
                    })?
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "format fields tidak terduga pada object {object_id}"
                )
                .into())
            }
        };

        let metadata: IotaLogMetadata = serde_json::from_str(json_data_str)
            .map_err(|e| {
                anyhow::anyhow!("gagal parse IotaLogMetadata dari object {object_id}: {e}")
            })?;

        Ok(LogRecordOnChain { object_id, metadata })
    }

    // ── hash_file ─────────────────────────────────────────────────────────────

    pub async fn hash_file(file_path: &str) -> Result<String, AuditError> {
        let bytes = fs::read(file_path)
            .await
            .map_err(|e| anyhow::anyhow!("gagal baca file untuk hashing: {e}"))?;

        let mut hasher = Sha256::new();
        hasher.update(&bytes);

        Ok(hex::encode(hasher.finalize()))
    }
}
