use serde::{Serialize, Deserialize};

// ── Tipe data publik ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedAuditEvent {
    pub payload: String,
    pub signature: String,    // base64: IotaSignature
    pub public_key: String,   // hex: raw public key bytes
    pub iota_address: String, // untuk verifikasi langsung
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub source_component: String,
    pub actor: String,
    pub target_object: String,
    pub outcome: AuditOutcome,
    pub action_type: String,

    #[serde(flatten)]
    pub details: AuditEventDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditOutcome {
    Success,
    Failure,
    Denied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum AuditEventDetails {
    #[serde(rename = "EV1")]
    Authentication {
        auth_method: String,
        authentication_result: String,
        failed_attempt_count: u32,
        device_fingerprint: String,
    },

    #[serde(rename = "EV4")]
    MedicalRecordAccess {
        access_type: String,
        medical_record_id: String,
        capability_id: String,
        authorization_token_id: String,
    },

    #[serde(rename = "EV5")]
    HospitalPersonnelKeyGeneration {
        facility_id: String,
        personnel_id: String,
        activation_key_id: String,
    },

    #[serde(rename = "EV8")]
    IotaTransactionSubmission {
        transaction_digest: String,
        signer_identity: String,
        payload_hash: String,
        network_confirmation_status: String,
    },

    #[serde(rename = "EV9")]
    GasSponsorshipRequest {
        requested_gas_budget: u64,
        requester_id: String,
    },

}