use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use std::fmt::Debug;
use iota_json_rpc_types::{IotaObjectRef, IotaTransactionBlockEffects};
use iota_types::{
    base_types::IotaAddress,
};

use crate::iota_client::IotaLogMetadata;

#[derive(Debug, Deserialize, Serialize)]
pub struct SuccessResponse<T>
where
    T: Debug,
{
    pub data: T,
    pub status_code: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UtilIpfsAddResponse {
    pub allocations: Vec<String>,
    pub cid: String,
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ReserveGasResponse {
    pub error: Option<String>,
    pub result: Option<ReserveGasResult>,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ReserveGasResult {
    pub gas_coins: Vec<IotaObjectRef>,
    pub reservation_id: u64,
    pub sponsor_address: IotaAddress,
}

// ← Tambahkan Clone agar .clone() bisa dipanggil di iota_client.rs
#[derive(Debug, Clone, Deserialize, JsonSchema, Serialize)]
pub struct ExecuteTxResponse {
    pub effects: Option<IotaTransactionBlockEffects>,
    pub error: Option<String>,
}

/// Satu file audit yang dikumpulkan selama rolling window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditBatch {
    pub batch_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub record_count: u64,
    pub first_record_id: Uuid,
    pub last_record_id: Uuid,
    pub first_record_hash: String,
    pub final_record_hash: String,
    pub file_hash: String,
    pub ipfs_cid: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedSignedEvent {
    pub enc_aes_key: String,   // ECIES: ephemeral_pubkey.wrap_nonce.enc_key (base64)
    pub ciphertext: String,    // base64: AuditEvent terenkripsi AES-256-GCM
    pub nonce: String,         // base64: nonce AES-GCM
    pub signature: String,     // base64: IotaSignature atas ciphertext bytes
    pub iota_address: String,  // untuk verifikasi signature
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum AuditOutcome {
    Success,
    Failure,
    Denied,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum AuditSourceComponent {
    HospitalClient,
    PatientClient,
    MinistryClient,
    ProxyReencryption,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditActionType {
    Create,
    Read,
    Update,
    Delete,
    Validate,
    Execute,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum AuditActorType {
    Pasien,
    PersonnelMedisFasyankes,
    PersonnelAdministratifFasyankes,
    AdminFasyankes,
    Kementerian,
    PREServer,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum AuditTargetObjectType {
    ActivationKey,
    AccessCapability,
    MedicalRecord,
    MedicalRecordMetadata,
    AccessDelegationQR,
    AdministrativeData,
    Nonce,
    AccessKeys,
    Transaction
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuditEvent {
    pub source_component: AuditSourceComponent,
    pub source_timestamp: DateTime<Utc>,
    pub event: Event
}

pub struct Event {
    pub actor_id: String,
    pub actor_type: AuditActorType,
    pub target_object_type: AuditTargetObjectType,
    pub target_object: String,
    pub outcome: AuditOutcome,
    pub action_type: AuditActionType,

    #[serde(flatten)]
    pub details: AuditEventDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub record_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub prev_record_hash: Option<String>,
    pub record_hash: String,
    
    #[serde(flatten)] 
    pub event: EncryptedSignedEvent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "event_type")]
pub enum AuditEventDetails {
    #[serde(rename = "EV1")]
    Authentication {
        auth_method: String,
        role: String,
        attempt_count: u32,
        failure_reason: Option<String>,
        device_info: Option<String>,
    },
    #[serde(rename = "EV2")]
    QrValidation {
        qr_content_hash: String,
        hospital_personnel_iota_address: String,
        hospital_personnel_pre_public_key_fingerprint: String,
        hospital_name: String,
        hospital_personnel_name: String,
        validation_step: String,
    },
    #[serde(rename = "EV3")]
    CapabilityCreation {
        access_type: String,
        exp_duration_ms: u64,
        k_frag_fingerprint: String,
        transaction_digest: String,
        hospital_name: String,
        nonce_used: String,
    },
    #[serde(rename = "EV4")]
    MedicalRecordAccess {
        patient_iota_address: String,
        record_index: Option<u64>,
        role_used: String,
        purpose_used: String,
        jwt_sub: String,
        capability_valid: bool,
        ipfs_cid: Option<String>,
        reencryption_performed: bool,
    },
    #[serde(rename = "EV5")]
    PersonnelActivationKey {
        admin_iota_address: String,
        personnel_id_hash: String,
        role_assigned: String,
        hospital_id_hash: String,
        transaction_digest: String,
        activation_key_hash: String,
    },
    #[serde(rename = "EV6")]
    FacilityRegistration {
        facility_id: String,
        facility_name: String,
        administrator_id: String,
        transaction_digest: String,
    },
    #[serde(rename = "EV7")]
    PreServiceOperation {
        endpoint_called: String,
        request_id: String,
        caller_component: String,
        channel_encryption: String,
        jwt_purpose: Option<String>,
        http_status_code: u16,
        latency_ms: u64,
    },
    #[serde(rename = "EV8")]
    IotaTransaction {
        transaction_digest: String,
        payload_hash: String,
        move_function: String,
        move_module: String,
        network_confirmation_status: String,
        gas_used: Option<u64>,
        sponsor_address: String,
    },
    #[serde(rename = "EV9")]
    GasSponsorship {
        gas_budget_requested: u64,
        reserve_duration_secs: u64,
        reservation_id: Option<u64>,
        sponsor_address: Option<String>,
        transaction_digest: Option<String>,
        gas_coin_object_ids: Vec<String>,
    },
    #[serde(rename = "EV10")]
    RedisOperation {
        redis_key_type: String,
        operation_type: String,
        ttl_remaining: Option<i64>,
        key_pattern: String,
    },
    #[serde(rename = "EV11")]
    IpfsOperation {
        cid: String,
        operation_type: String,
        data_size: Option<u64>,
        patient_iota_address: Option<String>,
        ipfs_node_url: String,
    },
    #[serde(rename = "EV12")]
    IotaMetadataOperation {
        object_id: String,
        object_type: String,
        move_function: String,
        is_mutable: bool,
        transaction_digest: Option<String>,
        dev_inspect_used: bool,
    },
    #[serde(rename = "EV13")]
    CapabilityValidation {
        capability_id: String,
        required_scope: String,
        actual_scope: String,
        rejection_reason: Option<String>,
        middleware_layer: String,
    },
}

impl AuditEventDetails {
    pub fn event_type(&self) -> &'static str {
        match self {
            AuditEventDetails::Authentication { .. } => "EV1",
            AuditEventDetails::QrValidation { .. } => "EV2",
            AuditEventDetails::CapabilityCreation { .. } => "EV3",
            AuditEventDetails::MedicalRecordAccess { .. } => "EV4",
            AuditEventDetails::PersonnelActivationKey { .. } => "EV5",
            AuditEventDetails::FacilityRegistration { .. } => "EV6",
            AuditEventDetails::PreServiceOperation { .. } => "EV7",
            AuditEventDetails::IotaTransaction { .. } => "EV8",
            AuditEventDetails::GasSponsorship { .. } => "EV9",
            AuditEventDetails::RedisOperation { .. } => "EV10",
            AuditEventDetails::IpfsOperation { .. } => "EV11",
            AuditEventDetails::IotaMetadataOperation { .. } => "EV12",
            AuditEventDetails::CapabilityValidation { .. } => "EV13",
        }
    }
}

// ── GET /api/logs ──────────────────────────────────────────────────────────

/// Query params for `GET /api/logs`.
///
/// `cursor` is an opaque string (the previous response's `next_cursor`,
/// which is an IOTA ObjectID in hex form) — pass it back unchanged to get
/// the next page. Omit it to fetch the first page.
// Ganti GetLogsQueryParams yang lama:
#[derive(Debug, Deserialize)]
pub struct GetLogsQueryParams {
    /// Cursor berbasis offset (indeks awal), bukan ObjectID.
    pub cursor: Option<u64>,
    pub limit: Option<u64>,
}

// Response per record
#[derive(Debug, Serialize)]
pub struct ApiLogRecord {
    pub index: u64,
    pub json_data: String,        // raw JSON metadata
    pub metadata: IotaLogMetadata, // sudah di-parse
}

// Response body GET /api/logs
#[derive(Debug, Serialize)]
pub struct GetLogsResponse {
    pub data: Vec<ApiLogRecord>,
    pub total: u64,
    pub cursor: u64,
    pub has_next_page: bool,
}
