use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use iota_json_rpc_types::{IotaTransactionBlockEffectsAPI, IotaObjectDataOptions};
use iota_types::{
    base_types::{IotaAddress, ObjectID},
    crypto::{IotaKeyPair, Signature},
    transaction::{CallArg, Transaction},
    Identifier,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shared_crypto::intent::{Intent, IntentMessage};
use std::str::FromStr;

use crate::audit_error::AuditError;
use crate::constants::{
    AUDIT_LOG_STORE_INITIAL_SHARED_VERSION, AUDIT_LOG_STORE_OBJECT_ID, IOTA_KEY_PAIR,
};
use crate::iota_utils::IotaUtils;

// ── Tipe metadata yang disimpan on-chain ──────────────────────────────────────

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

// ── Hasil satu record yang dibaca dari store ──────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecordOnChain {
    pub index: u64,
    pub json_data: String,
    pub metadata: IotaLogMetadata,
}

// ── Hasil publish ─────────────────────────────────────────────────────────────

pub struct PublishResult {
    pub object_id: ObjectID,
    pub tx_digest: String,
}

// ── Hasil paginasi ────────────────────────────────────────────────────────────

pub struct LogRecordsPage {
    pub records: Vec<LogRecordOnChain>,
    pub total: u64,
    pub has_next_page: bool,
}

// ── IotaLogClient ─────────────────────────────────────────────────────────────

pub struct IotaLogClient {
    key_pair: IotaKeyPair,
    pub package_id: ObjectID,
    pub store_id: ObjectID,
}

impl IotaLogClient {
    pub fn new(package_id_hex: &str) -> Result<Self, AuditError> {
        let key_pair = IotaKeyPair::decode(IOTA_KEY_PAIR)
            .map_err(|e| AuditError::from(anyhow::anyhow!("gagal decode keypair: {e}")))?;

        let package_id = ObjectID::from_hex_literal(package_id_hex)
            .map_err(|e| AuditError::from(anyhow::anyhow!("Package ID tidak valid: {e}")))?;

        let store_id = ObjectID::from_hex_literal(AUDIT_LOG_STORE_OBJECT_ID)
            .map_err(|e| AuditError::from(anyhow::anyhow!("Store ID tidak valid: {e}")))?;

        Ok(Self {
            key_pair,
            package_id,
            store_id,
        })
    }

    // ── Ambil address dari keypair ────────────────────────────────────────────

    fn sender(&self) -> IotaAddress {
        (&self.key_pair.public()).into()
    }

    // ── Bangun CallArg untuk shared store ─────────────────────────────────────

    fn store_call_arg(&self) -> CallArg {
        IotaUtils::construct_shared_object_call_arg(
            self.store_id,
            AUDIT_LOG_STORE_INITIAL_SHARED_VERSION,
            true, // mutable
        )
    }

    // =========================================================================
    // publish_metadata — sama seperti sebelumnya
    // =========================================================================

    pub async fn publish_metadata(
        &self,
        metadata: &IotaLogMetadata,
    ) -> Result<PublishResult, AuditError> {
        let metadata_json = serde_json::to_string(metadata)
            .map_err(|e| anyhow::anyhow!("gagal serialize metadata: {e}"))?;

        let iota_client = IotaUtils::get_iota_client().await?;
        let sender = self.sender();

        let module = Identifier::new("audit_log")
            .map_err(|e| anyhow::anyhow!("nama module tidak valid: {e}"))?;

        let call_args = vec![
            self.store_call_arg(),
            CallArg::Pure(
                bcs::to_bytes(&metadata_json)
                    .map_err(|e| anyhow::anyhow!("gagal encode BCS: {e}"))?,
            ),
        ];

        let pt = IotaUtils::construct_pt(
            "create_log".to_string(),
            self.package_id,
            module,
            vec![],
            call_args,
        )?;

        let (sponsor_address, reservation_id, gas_coins) =
            IotaUtils::reserve_gas(10_000_000, 60).await?;
        let ref_gas_price = IotaUtils::get_ref_gas_price(&iota_client).await?;

        let tx_data = IotaUtils::construct_sponsored_tx_data(
            sender,
            gas_coins,
            pt,
            10_000_000,
            ref_gas_price,
            sponsor_address,
        );

        let intent_msg = IntentMessage::new(Intent::iota_transaction(), &tx_data);
        let signature = Signature::new_secure(&intent_msg, &self.key_pair);
        let tx = Transaction::from_data(tx_data, vec![signature]);

        let exec_response = IotaUtils::execute_tx(tx, reservation_id).await?;

        if let Some(err) = exec_response.error {
            return Err(anyhow::anyhow!("execute_tx error: {err}").into());
        }

        let effects = exec_response
            .effects
            .ok_or_else(|| anyhow::anyhow!("effects tidak tersedia"))?;

        let tx_digest = effects.transaction_digest().to_string();

        println!("[iota] publish_metadata OK | TX: {tx_digest} | status: {0}", effects.status());

        Ok(PublishResult {
            object_id: self.store_id,
            tx_digest,
        })
    }

    // =========================================================================
    // get_total_records — memanggil record_count() via dev_inspect
    // =========================================================================

    pub async fn get_total_records(&self) -> Result<u64, AuditError> {
        let iota_client = IotaUtils::get_iota_client().await?;
        let sender = self.sender();

        let module = Identifier::new("audit_log")
            .map_err(|e| anyhow::anyhow!("module identifier tidak valid: {e}"))?;

        let pt = IotaUtils::construct_pt(
            "record_count".to_string(),
            self.package_id,
            module,
            vec![],
            vec![self.store_call_arg()],
        )?;

        let response = IotaUtils::move_call_read_only(sender, &iota_client, pt).await?;
        IotaUtils::handle_error_move_call_read_only(response.clone())?;

        let total: u64 = IotaUtils::parse_move_read_only_result(response, 0)?;
        Ok(total)
    }

    // =========================================================================
    // list_log_records — IMPLEMENTASI UTAMA (dev_inspect + offset cursor)
    // =========================================================================

    /// Ambil satu halaman LogRecord dari AuditLogStore via dev_inspect.
    ///
    /// - `cursor`: indeks offset awal (0-based). `None` → mulai dari 0.
    /// - `limit`: maksimum record per halaman (dikap oleh handler).
    pub async fn list_log_records(
        &self,
        cursor: Option<u64>,
        limit: u64,
    ) -> Result<LogRecordsPage, AuditError> {
        let iota_client = IotaUtils::get_iota_client().await?;
        let sender = self.sender();

        let offset = cursor.unwrap_or(0);

        let module = Identifier::new("audit_log")
            .map_err(|e| anyhow::anyhow!("module identifier tidak valid: {e}"))?;

        // Panggil get_logs(store, cursor, limit) di Move
        let pt = IotaUtils::construct_pt(
            "get_logs".to_string(),
            self.package_id,
            module,
            vec![],
            vec![
                self.store_call_arg(),
                CallArg::Pure(
                    bcs::to_bytes(&offset)
                        .map_err(|e| anyhow::anyhow!("gagal encode cursor BCS: {e}"))?,
                ),
                CallArg::Pure(
                    bcs::to_bytes(&limit)
                        .map_err(|e| anyhow::anyhow!("gagal encode limit BCS: {e}"))?,
                ),
            ],
        )?;

        let response = IotaUtils::move_call_read_only(sender, &iota_client, pt).await?;
        IotaUtils::handle_error_move_call_read_only(response.clone())?;

        // Return value index 0 → vector<LogRecord> (BCS encoded)
        // Asumsikan LogRecord di Move berisi satu field String: json_data
        let raw_vec: Vec<String> =
            IotaUtils::parse_move_read_only_result(response.clone(), 0)?;

        let mut records = Vec::with_capacity(raw_vec.len());
        for (i, json_data) in raw_vec.into_iter().enumerate() {
            let global_index = offset + i as u64;
            let metadata: IotaLogMetadata =
                serde_json::from_str(&json_data).map_err(|e| {
                    anyhow::anyhow!(
                        "gagal parse IotaLogMetadata di index {global_index}: {e}"
                    )
                })?;
            records.push(LogRecordOnChain {
                index: global_index,
                json_data,
                metadata,
            });
        }

        // Ambil total agar bisa hitung has_next_page
        let total = self.get_total_records().await.unwrap_or(0);
        let has_next_page = offset + limit < total;

        Ok(LogRecordsPage {
            records,
            total,
            has_next_page,
        })
    }

    // =========================================================================
    // Fallback: baca store object langsung (jika Move function belum tersedia)
    // =========================================================================

    /// Alternatif tanpa Move function view — baca field `records` dari
    /// content object AuditLogStore secara langsung.
    /// Cocok jika struktur store menggunakan `vector<String>` biasa (bukan Table).
    pub async fn list_log_records_via_object(
        &self,
        cursor: Option<u64>,
        limit: u64,
    ) -> Result<LogRecordsPage, AuditError> {
        let iota_client = IotaUtils::get_iota_client().await?;

        let response = iota_client
            .read_api()
            .get_object_with_options(
                self.store_id,
                IotaObjectDataOptions {
                    show_content: true,
                    show_type: true,
                    show_owner: true,
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| anyhow::anyhow!("gagal fetch store object: {e}"))?;

        let object_data = response
            .data
            .ok_or_else(|| anyhow::anyhow!("object data kosong"))?;

        let content = object_data
            .content
            .ok_or_else(|| anyhow::anyhow!("content object kosong"))?;

        let move_object = match content {
            iota_json_rpc_types::IotaParsedData::MoveObject(obj) => obj,
            _ => return Err(anyhow::anyhow!("bukan MoveObject").into()),
        };

        // Ekstrak field "records" (vector<String>)
        let all_json_strings: Vec<String> = match &move_object.fields {
            iota_json_rpc_types::IotaMoveStruct::WithFields(fields) => {
                let records_value = fields
                    .get("records")
                    .ok_or_else(|| anyhow::anyhow!("field 'records' tidak ditemukan"))?;

                match records_value {
                    iota_json_rpc_types::IotaMoveValue::Vector(vec_values) => vec_values
                        .iter()
                        .filter_map(|v| {
                            if let iota_json_rpc_types::IotaMoveValue::String(s) = v {
                                Some(s.clone())
                            } else {
                                None
                            }
                        })
                        .collect(),
                    _ => {
                        return Err(anyhow::anyhow!(
                            "field 'records' bukan Vector"
                        )
                        .into())
                    }
                }
            }
            _ => {
                return Err(
                    anyhow::anyhow!("format fields object tidak terduga").into()
                )
            }
        };

        let total = all_json_strings.len() as u64;
        let offset = cursor.unwrap_or(0);

        let slice: Vec<String> = all_json_strings
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect();

        let mut records = Vec::with_capacity(slice.len());
        for (i, json_data) in slice.into_iter().enumerate() {
            let global_index = offset + i as u64;
            let metadata: IotaLogMetadata = serde_json::from_str(&json_data)
                .map_err(|e| anyhow::anyhow!("gagal parse metadata index {global_index}: {e}"))?;
            records.push(LogRecordOnChain {
                index: global_index,
                json_data,
                metadata,
            });
        }

        let has_next_page = offset + limit < total;

        Ok(LogRecordsPage {
            records,
            total,
            has_next_page,
        })
    }

    // =========================================================================
    // hash_file — tidak berubah
    // =========================================================================

    pub async fn hash_file(file_path: &str) -> Result<String, AuditError> {
        use tokio::fs;
        let bytes = fs::read(file_path)
            .await
            .map_err(|e| anyhow::anyhow!("gagal baca file: {e}"))?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        Ok(hex::encode(hasher.finalize()))
    }
}