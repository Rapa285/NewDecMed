use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::types::{HospitalPersonnelRole};
use std::str::FromStr;
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
    SignIn,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum AuditActorType {
    AdministrativePersonnel,
    MedicalPersonnel,
    Patient,
    Admin,
    Ministry,
    PREServer,
    Unknown,
    Client,
}

impl From<HospitalPersonnelRole> for AuditActorType {
    fn from(role: HospitalPersonnelRole) -> Self {
        match role {
            HospitalPersonnelRole::Admin => AuditActorType::Admin,
            HospitalPersonnelRole::AdministrativePersonnel => AuditActorType::AdministrativePersonnel,
            HospitalPersonnelRole::MedicalPersonnel => AuditActorType::MedicalPersonnel,
        }
    }
}

impl FromStr for AuditActorType {
    type Err = String; // Anda bisa mengganti ini dengan custom error type

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "AdministrativePersonnel" => Ok(AuditActorType::AdministrativePersonnel),
            "MedicalPersonnel" => Ok(AuditActorType::MedicalPersonnel),
            "Patient" => Ok(AuditActorType::Patient),
            "Admin" => Ok(AuditActorType::Admin),
            "Ministry" => Ok(AuditActorType::Ministry),
            "PREServer" => Ok(AuditActorType::PREServer),
            _ => Err(format!("'{}' bukan tipe AuditActorType yang valid", s)),
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
    GasReservation,
    Account,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum AuditEventDetails {
    #[serde(rename = "EV1")]
    Authentication {
        auth_method: String,
        role: String,
        failure_reason: Option<String>,
    },

    #[serde(rename = "EV3")]
    CapabilityCreation {
        access_type: String,
        transaction_digest: String,
        receiver: String,
        nonce_used: String,
    },

    #[serde(rename = "EV4")]
    MedicalRecordAccess {
        patient_iota_address: String,
        record_index: Option<u64>,
        role_used: String,
    },
    
    #[serde(rename = "EV5")]
    PersonnelActivationKey {
        admin_iota_address: String,
        personnel_id: String,
        personnel_id_hash: String,
        role_assigned: String,
        facility_id: String,
        activation_key_id: String,
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