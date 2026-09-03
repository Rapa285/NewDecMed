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
    #[serde(rename = "EV6")]
    PRERequest {
        endpoint_called: String,
        request_id: String,
        caller_component: String,
        channel_encryption: String,
    },

    #[serde(rename = "EV7")]
    Reencryption {
        reencryption_operation_id: String,
        capability_id: String,
        target_ciphertext: String,
        kfrag_identifier: String,
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

    #[serde(rename = "EV13")]
    RedisWrite {
        redis_key_type: String,
        operation_type: String,
        ttl_remaining: i64,
    },

    #[serde(rename = "EV14")]
    RedisRead {
        redis_key_type: String,
        request_origin: String,
        ttl_remaining: i64,
    },

    #[serde(rename = "EV15")]
    IPFSOperation {
        cid: String,
        operation_type: String,
        data_size: u64,
    },

    #[serde(rename = "EV16")]
    IPFSVerification {
        cid: String,
        expected_hash: String,
        verification_result: bool,
    },

    #[serde(rename = "EV17")]
    LedgerQuery {
        metadata_type: String,
        object_id: String,
        query_requester_id: String,
    },

    #[serde(rename = "EV18")]
    CapabilityValidation {
        capability_checked: String,
        required_scope: String,
        actual_scope: String,
        validation_result: bool,
    },
}