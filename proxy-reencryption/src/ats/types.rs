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
}