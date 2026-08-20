use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use std::fmt::Debug;
use iota_json_rpc_types::{IotaObjectRef, IotaTransactionBlockEffects};
use iota_types::{
    base_types::IotaAddress,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct SuccessResponse<T>
where
    T: Debug,
{
    pub data: T,
    pub status_code: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UtilIpfsAddResponse {
    pub allocations: Vec<String>,
    pub cid: String,
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ReserveGasResponse {
    pub error: Option<String>,
    pub result: Option<ReserveGasResult>,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ReserveGasResult {
    pub gas_coins: Vec<IotaObjectRef>,
    pub reservation_id: u64,
    pub sponsor_address: IotaAddress,
}

// ← Tambahkan Clone agar .clone() bisa dipanggil di iota_client.rs
#[derive(Debug, Clone, Deserialize, JsonSchema, Serialize)]
pub struct ExecuteTxResponse {
    pub effects: Option<IotaTransactionBlockEffects>,
    pub error: Option<String>,
}


/// Representasi hasil dari pelaksanaan audit event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditOutcome {
    Success,
    Failure,
    Denied,
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct SignedEvent {
    pub payload: String,
    pub signature: String,
    pub public_key: String,
}

impl SignedEvent {
    pub fn canonical_message(&self) -> Result<String, serde_json::Error> {
        let payload_str = serde_json::to_string(&self.payload)?;
        Ok(format!(
            "{}|{}|{}",
            payload_str,
            self.signature,
            self.public_key
        ))
    }
}

/// Satu file audit yang dikumpulkan selama rolling window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditBatch {
    pub batch_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub record_count: u64,
    pub first_record_id: Uuid,
    pub last_record_id: Uuid,
    pub first_record_hash: String,
    pub final_record_hash: String,
    pub file_hash: String,
    pub ipfs_cid: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub record_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub prev_record_hash: Option<String>,
    pub record_hash: String,
    
    #[serde(flatten)] 
    pub event: AuditEvent,
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

    #[serde(rename = "EV4")]
    MedicalRecordAccess {
        access_type: String,
        medical_record_id: String,
        capability_id: String,
        authorization_token_id: String,
    },

    #[serde(rename = "EV5")]
    FacilityRegistration {
        facility_id: String,
        facility_name: String,
        administrator_id: String,
        transaction_digest: String,
    },

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