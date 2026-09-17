use serde::{Serialize, Deserialize};

// ── Tipe data publik ──────────────────────────────────────────────────────────

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct SignedAuditEvent {
//     pub payload: String,
//     pub signature: String,
//     pub iota_address: String,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedAuditEvent {
    pub enc_key: String,       // base64: AES key terenkripsi
    pub ciphertext: String,    // base64: payload terenkripsi
    pub nonce: String,         // base64: nonce AES-GCM
    pub signature: String,     // hex: sign(ciphertext_bytes)
    pub public_key: String,    // hex: verifying key aktor
    pub iota_address: String,  // untuk lookup binding
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
    #[serde(rename = "EV7")]
    PREServiceRequest {
        endpoint_called: String,
        request_id: String,
        caller_component: String,
        channel_encryption: String,
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

    #[serde(rename = "EV10")]
    RedisOperation {
        redis_key_type: String,
        operation_type: String,
        ttl_remaining: i64,
    },

    #[serde(rename = "EV11")]
    IPFSObjectAccess {
        cid: String,
        operation_type: String,
        requester_id: String,
    },

    #[serde(rename = "EV12")]
    OnChainMetadataAccess {
        onchain_object_id: String,
        requester_id: String,
    },

    #[serde(rename = "EV13")]
    CapabilityValidation {
        capability_id: String,
        requester_id: String,
        validation_result: bool,
        rejection_reason: Option<String>,
    },
}

impl AuditEventDetails {
    pub fn event_type(&self) -> &'static str {
        match self {
            AuditEventDetails::PREServiceRequest { .. } => "EV7",
            AuditEventDetails::IotaTransactionSubmission { .. } => "EV8",
            AuditEventDetails::GasSponsorshipRequest { .. } => "EV9",
            AuditEventDetails::RedisOperation { .. } => "EV10",
            AuditEventDetails::IPFSObjectAccess { .. } => "EV11",
            AuditEventDetails::OnChainMetadataAccess { .. } => "EV12",
            AuditEventDetails::CapabilityValidation { .. } => "EV13",
        }
    }
}

impl AuditEvent {
    pub fn event_type(&self) -> &'static str {
        self.details.event_type()
    }
}