use serde::{Serialize, Deserialize};

// ── Tipe data publik ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedAuditEvent {
    pub payload: String,
    pub signature: String,
    pub public_key: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    #[serde(rename = "EV12")]
    IotaMetadataOperation {
        object_id: String,
        object_type: String,
        move_function: String,
        is_mutable: bool,
        transaction_digest: Option<String>,
    },
}