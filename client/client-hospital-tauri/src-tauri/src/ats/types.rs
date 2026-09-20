use serde::{Serialize, Deserialize};

// ── Tipe data publik ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedAuditEvent {
    pub payload: String,
    pub signature: String,    // base64: IotaSignature
    pub public_key: String,   // hex: raw public key bytes
    pub iota_address: String, // untuk verifikasi langsung
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

}