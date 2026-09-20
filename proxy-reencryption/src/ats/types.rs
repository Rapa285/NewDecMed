use serde::{Serialize, Deserialize};
use crate::types::AuthRole;
use chrono::{DateTime, Utc};

// ── Encrypted transport wrapper ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSignedEvent {
    pub enc_aes_key: String,   // base64: ECIES(ephemeral_pubkey.wrap_nonce.enc_key)
    pub ciphertext: String,    // base64: AuditEvent terenkripsi AES-256-GCM
    pub nonce: String,         // base64: nonce AES-GCM (12 bytes)
    pub signature: String,     // base64: IotaSignature atas ciphertext bytes
    pub iota_address: String,  // untuk verifikasi signature di ATS server
}

// ── Enums (harus cocok dengan audit-trail/src/types.rs) ──────────────────────

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
    AdministrativePersonnel,
    MedicalPersonnel,
    Patient,
    Admin,
    Ministry,
    PREServer,
}

impl From<AuthRole> for AuditActorType {
    fn from(role: AuthRole) -> Self {
        match role {
            AuthRole::AdministrativePersonnel => AuditActorType::AdministrativePersonnel,
            AuthRole::MedicalPersonnel => AuditActorType::MedicalPersonnel,
            AuthRole::Patient => AuditActorType::Patient,
        }
    }
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
    IPFSObject,
    GasReservation,
    PREEndPoint,
    OnChainData,
}

// ── AuditEvent ────────────────────────────────────────────────────────────────

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

// ── AuditEventDetails (harus cocok dengan audit-trail/src/types.rs) ───────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum AuditEventDetails {
    /// EV7 - PRE Service Request/Response
    #[serde(rename = "EV7")]
    PreServiceOperation {
        endpoint_called: String,
        request_id: String,
        caller_component: String,
        channel_encryption: String,
        jwt_purpose: Option<String>,
    },

    /// EV8 - IOTA Transaction Submission
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

    /// EV9 - Gas Sponsorship Request
    #[serde(rename = "EV9")]
    GasSponsorship {
        gas_budget_requested: u64,
        reserve_duration_secs: u64,
        reservation_id: Option<u64>,
        sponsor_address: Option<String>,
        transaction_digest: Option<String>,
        gas_coin_object_ids: Vec<String>,
    },

    /// EV10 - Redis Operation
    #[serde(rename = "EV10")]
    RedisOperation {
        redis_key_type: String,
        ttl_remaining: Option<i64>,
        key_pattern: String,
    },

    /// EV11 - IPFS Object Access
    #[serde(rename = "EV11")]
    IpfsOperation {
        data: Option<String>,
        data_size: Option<u64>,
        ipfs_node_url: String,
    },

    /// EV12 - On-chain Metadata Access
    #[serde(rename = "EV12")]
    IotaMetadataOperation {
        object_id: String,
        object_type: String,
        move_function: String,
        is_mutable: bool,
        transaction_digest: Option<String>,
        requester_id: String,
    },

    /// EV13 - Capability Validation
    #[serde(rename = "EV13")]
    CapabilityValidation {
        capability_id: String,
        requester_id: String,
        rejection_reason: Option<String>,
        validation_result: bool,
    },
}

impl AuditEventDetails {
    pub fn event_type(&self) -> &'static str {
        match self {
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