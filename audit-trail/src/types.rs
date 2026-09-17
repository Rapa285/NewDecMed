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

#[derive(Debug, Deserialize)]
pub struct SignedEvent {
    pub payload: String,
    pub signature: String,    // base64: IotaSignature
    pub iota_address: String, // wajib, untuk verifikasi
}

impl SignedEvent {
    pub fn canonical_message(&self) -> Result<String, serde_json::Error> {
        let payload_str = serde_json::to_string(&self.payload)?;
        Ok(format!(
            "{}|{}|{}",
            payload_str,
            self.signature,
            self.iota_address
        ))
    }
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
    Signin,
    Signup,
    Signout,
    ValidatePin,
    ValidateSeedWords,
    UseActivationKey,
    QrScan,
    QrDecode,
    QrValidate,
    CreateAccess,
    StoreKeys,
    RevokeAccess,
    ReadMedicalRecord,
    CreateMedicalRecord,
    UpdateMedicalRecord,
    ReadAdministrativeData,
    CreatePersonnelActivationKey,
    UpdatePersonnelActivationKey,
    CreateFacilityActivationKey,
    UpdateFacilityActivationKey,
    NonceRequest,
    PreReencrypt,
    JwtIssue,
    AuthMiddlewareCheck,
    IotaTransaction,
    GasReserve,
    GasExecute,
    RedisGet,
    RedisSet,
    RedisDel,
    IpfsUpload,
    IpfsFetch,
    ChainRead,
    ChainWrite,
    JwtValidate,
    RoleCheck,
    PurposeCheck,
    ProxyCapValidate,
    ScopeValidate,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuditEvent {
    pub source_component: AuditSourceComponent,
    pub actor: String,
    pub target_object: String,
    pub outcome: AuditOutcome,
    pub action_type: AuditActionType,
    pub details: AuditEventDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub record_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub prev_record_hash: Option<String>,
    pub record_hash: String,
    
    #[serde(flatten)] 
    pub event: AuditEvent,
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
        activation_key_hash: String,
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
#[derive(Debug, Deserialize)]
pub struct GetLogsQueryParams {
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}

/// A single row in the `GET /api/logs` response.
#[derive(Debug, Serialize)]
pub struct ApiLogRecord {
    pub object_id: String,
    pub metadata: IotaLogMetadata,
}

/// Response body for `GET /api/logs`. Shape matches what the
/// `audit-trail-client` (Tauri) app expects.
#[derive(Debug, Serialize)]
pub struct GetLogsResponse {
    pub data: Vec<ApiLogRecord>,
    pub next_cursor: Option<String>,
    pub has_next_page: bool,
}
