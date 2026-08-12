
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct UtilIpfsAddResponse {
    pub allocations: Vec<String>,
    pub cid: String,
    pub name: String,
    pub size: u64,
}   


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    QrAccessDelegationValidation,
    CapabilityCreation,
    MedicalRecordAccess,
    ActivationKeyIssuance,
    HealthcareFacilityRegistration,
    PreServiceRequest,
    ReencryptionOperation,
    BlockchainTransactionSubmission,
    GasSponsorshipRequest,
    GasSponsorshipDecision,
    ClientKeyStoreOperation,
    ClientKeyStoreAccess,
    SensitiveCacheObjectOperation,
    SensitiveCacheObjectAccess,
    IpfsObjectAccess,
    IpfsObjectIntegrityVerification,
    OnchainMetadataAccess,
    CapabilityValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecordContent {
    pub event: EventRelatedContent,
    pub user: UserRelatedContent,
    pub audit_system: AuditSystemRelatedContent,
    pub participant_object: ParticipantObjectRelatedContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRelatedContent {
    pub event_id: String,
    pub event_action_code: String,
    pub event_date_time: String,
    pub event_outcome_indicator: String,
    pub event_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRelatedContent {
    pub user_id: String,
    pub user_name: String,
    pub user_role: String,
    pub is_requestor: bool,
    pub network_access_point: Option<String>,
    pub network_access_point_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSystemRelatedContent {
    pub audit_source_id: String,
    pub audit_source_name: Option<String>,
    pub audit_source_type: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantObjectRelatedContent {
    pub object_id: String,
    pub object_type: String,
    pub object_role: String,
    pub data_classification: Option<String>,
    pub description: Option<String>,
}