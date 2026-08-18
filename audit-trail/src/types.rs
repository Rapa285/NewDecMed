
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, Serialize)]
pub struct UtilIpfsAddResponse {
    pub allocations: Vec<String>,
    pub cid: String,
    pub name: String,
    pub size: u64,
}


// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// pub enum EventType {
//     Authentication
//     QrAccessDelegationValidation,
//     CapabilityCreation,
//     MedicalRecordAccess,
//     ActivationKeyIssuance,
//     HealthcareFacilityRegistration,
//     PreServiceRequest,
//     ReencryptionOperation,
//     BlockchainTransactionSubmission,
//     GasSponsorshipRequest,
//     GasSponsorshipDecision,
//     ClientKeyStoreOperation,
//     ClientKeyStoreAccess,
//     SensitiveCacheObjectOperation,
//     SensitiveCacheObjectAccess,
//     IpfsObjectAccess,
//     IpfsObjectIntegrityVerification,
//     OnchainMetadataAccess,
//     CapabilityValidation,
// }

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
    pub source_id: String,
    pub payload: AuditEvent,
    pub timestamp: DateTime<Utc>,
    pub nonce: String,           
    pub signature: String, 
}

impl SignedEvent {
    pub fn canonical_message(&self) -> Result<String, serde_json::Error> {
        let payload_str = serde_json::to_string(&self.payload)?;
        Ok(format!(
            "{}|{}|{}|{}",
            self.source_id,
            self.timestamp.to_rfc3339(),
            self.nonce,
            payload_str,
        ))
    }
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
        expiry_duration: u64, // Bisa juga menggunakan tipe Duration
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
    ActivationKeyIssuance {
        activation_key_id: String,
        issuer_id: String,
        recipient_id: String,
        expiration_time: DateTime<Utc>,
    },

    #[serde(rename = "EV6")]
    FacilityRegistration {
        facility_id: String,
        facility_name: String,
        administrator_id: String,
        transaction_digest: String,
    },

    #[serde(rename = "EV7")]
    PRERequest {
        endpoint_called: String,
        request_id: String,
        caller_component: String,
        channel_encryption: String,
    },

    #[serde(rename = "EV8")]
    Reencryption {
        reencryption_operation_id: String,
        capability_id: String,
        target_ciphertext: String,
        kfrag_identifier: String,
    },

    #[serde(rename = "EV9")]
    IotaTransaction {
        transaction_digest: String,
        payload_hash: String,
        network_confirmation_status: String, // Atau enum Confirmed/Pending
    },

    #[serde(rename = "EV10")]
    SponsorshipRequest {
        requested_gas_budget: u64,
        requester_id: String,
        transaction_digest: String,
    },

    #[serde(rename = "EV11")]
    SponsorshipDecision {
        approval_status: String, // Atau bool (true/false)
        decision_reason: String,
        approved_by: String,
    },

    #[serde(rename = "EV12")]
    KeyManagement {
        key_operation: String, // Create, Update, Delete
        key_id: String,
        key_type: String,
    },

    #[serde(rename = "EV13")]
    KeyAccess {
        key_id: String,
        key_type: String,
        access_purpose: String,
    },

    #[serde(rename = "EV14")]
    RedisWrite {
        redis_key_type: String,
        operation_type: String,
        ttl_remaining: i64, // Dalam detik/milidetik
    },

    #[serde(rename = "EV15")]
    RedisRead {
        redis_key_type: String,
        request_origin: String,
        ttl_remaining: i64,
    },

    #[serde(rename = "EV16")]
    IPFSOperation {
        cid: String,
        operation_type: String, // Upload/Download
        data_size: u64, // Dalam bytes
    },

    #[serde(rename = "EV17")]
    IPFSVerification {
        cid: String,
        expected_hash: String,
        verification_result: bool,
    },

    #[serde(rename = "EV18")]
    LedgerQuery {
        metadata_type: String,
        object_id: String,
        query_requester_id: String,
    },

    #[serde(rename = "EV19")]
    CapabilityValidation {
        capability_checked: String,
        required_scope: String,
        actual_scope: String,
        validation_result: bool, // true jika valid, false jika tidak
    },
}