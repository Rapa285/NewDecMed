use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

// ── Tipe data publik ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSignedEvent {
    pub enc_aes_key: String,   // base64
    pub ciphertext: String,    // base64
    pub nonce: String,         // base64
    pub signature: String,     // base64: IotaSignature atas ciphertext bytes
    pub iota_address: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuditEvent {
    pub source_component: AuditSourceComponent,
    pub source_timestamp: DateTime<Utc>,
    pub event: Event
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
    Transaction,
    GasRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum AuditEventDetails {
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

impl Event {
    pub fn event_type(&self) -> &'static str {
        self.details.event_type()
    }
}