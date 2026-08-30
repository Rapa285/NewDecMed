use serde::{Serialize, Deserialize};

// ── Tipe data publik ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedAuditEvent {
    pub payload: String,
    pub signature: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub source_component: String,
    pub actor_id: String,
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

    #[serde(rename = "EV2")]
    QRDelegation {
        qr_payload_id: String,
        recipient_identity: String,
        signature_valid: bool,
    },

    #[serde(rename = "EV3")]
    CapabilityIssuance {
        capability_id: String,
        access_scope: String,
        expiry_duration: u64,
        transaction_digest: String,
    },

    #[serde(rename = "EV8")]
    IotaTransaction {
        transaction_digest: String,
        payload_hash: String,
        network_confirmation_status: String,
    },

    #[serde(rename = "EV9")]
    SponsorshipRequest {
        requested_gas_budget: u64,
        requester_id: String,
        transaction_digest: String,
    },

    #[serde(rename = "EV10")]
    SponsorshipDecision {
        approval_status: String,
        decision_reason: String,
        approved_by: String,
    },

    #[serde(rename = "EV11")]
    KeyManagement {
        key_operation: String,
        key_id: String,
        key_type: String,
    },

    #[serde(rename = "EV12")]
    KeyAccess {
        key_id: String,
        key_type: String,
        access_purpose: String,
    },

    #[serde(rename = "EV17")]
    LedgerQuery {
        metadata_type: String,
        object_id: String,
        query_requester_id: String,
    },
}