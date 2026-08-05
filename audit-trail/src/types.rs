
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive]
#[derive(Debug, Deserialize, Serialize)]
pub struct UtilIpfsAddResponse {
    pub allocations: Vec<String>,
    pub cid: String,
    pub name: String,
    pub size: u64,
}   

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct AuditRecord {
//     pub audit_record_id: String,

//     pub event: AuditEvent,

//     pub active_participants: Vec<ActiveParticipant>,

//     pub audit_source: AuditSourceIdentification,

//     pub participant_objects: Vec<ParticipantObjectIdentification>,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord{
    pub audit_record_id: Uuid,
    pub data : String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Coded identifier of the event type (e.g. "record-access",
    /// "record-create", "record-update", "record-archive", "user-login").
    pub event_id: String,

    /// Human-readable label for the event type.
    pub event_type: String,

    /// Action performed, aligned with typical audit action codes:
    /// Create, Read, Update, Delete, Execute.
    pub action_code: AuditActionCode,

    /// Date and time the event occurred (ISO 8601 / RFC 3339).
    pub event_datetime: String,

    /// Whether the event succeeded, failed, or was a minor/major failure.
    pub outcome_indicator: AuditOutcome,

    /// Optional free-text description of the outcome, e.g. error message.
    pub outcome_description: Option<String>,

    /// Purpose of use for the event (e.g. "treatment", "research",
    /// "emergency-override"), relevant for consent/authorization context.
    pub purpose_of_use: Option<String>,
}

/// Standard audit action codes (Create / Read / Update / Delete / Execute).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditActionCode {
    Create,
    Read,
    Update,
    Delete,
    Execute,
}

/// Outcome of the audited event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditOutcome {
    Success,
    MinorFailure,
    SeriousFailure,
    MajorFailure,
}

/// Category 2 — Active participant (actor) data.
/// Identifies *who* (human or system) took part in the event, and their
/// role/authorization context at the time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveParticipant {
    /// Unique identifier of the participant (user ID, service account ID,
    /// or device/system ID).
    pub participant_id: String,

    /// Display name of the participant (e.g. "Dr. Joan Smith").
    pub participant_name: Option<String>,

    /// Role(s) held by the participant during the event, e.g.
    /// "cardiologist", "medical-receptionist", "system-process".
    pub role: Vec<String>,

    /// Whether this participant is the one who *initiated* the event
    /// (true) or merely a secondary/delegated participant (false).
    pub is_requestor: bool,

    /// Network access point used by the participant (IP address,
    /// hostname, or terminal identifier).
    pub network_access_point: Option<String>,

    /// Type of network access point (e.g. "IPAddress", "MachineName").
    pub network_access_point_type: Option<String>,

    /// Authentication method/level used (e.g. "two-factor", "smart-card"),
    /// relevant to ISO 27799 §9.4.1 requirement on multi-factor auth.
    pub authentication_method: Option<String>,
}

/// Category 3 — Audit source identification.
/// Identifies *where* the audit record originated (which system/site
/// generated it), important when records are aggregated from multiple
/// distributed health information systems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSourceIdentification {
    /// Identifier of the source system/application that generated the
    /// audit record (e.g. hospital EHR system ID).
    pub audit_source_id: String,

    /// Human-readable name of the source (e.g. "RSUD-XYZ-EHR").
    pub audit_source_name: Option<String>,

    /// Type of source, e.g. "end-user-application", "webserver",
    /// "network-device", "healthcare-facility".
    pub audit_source_type: Vec<String>,

    /// Identifier of the site/facility from which the event originated,
    /// relevant for cross-jurisdictional/shared-care scenarios.
    pub site_id: Option<String>,
}

/// Category 4 — Participant object identification.
/// Identifies *what* was acted upon — typically the subject of care
/// (patient) and/or the specific health data/record involved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantObjectIdentification {
    /// Unique identifier of the object (e.g. patient ID, record ID,
    /// document ID).
    pub object_id: String,

    /// Type of the object being referenced.
    pub object_type: ParticipantObjectType,

    /// Role the object played in the event (e.g. "patient", "report",
    /// "prescription", "query-parameters").
    pub object_role: String,

    /// Sensitivity/classification of the object, e.g. "confidential"
    /// (per ISO 27799 §8.2.1, personal health information is uniformly
    /// classified as confidential).
    pub data_classification: Option<String>,

    /// Optional life-cycle stage of the object at the time of the event
    /// (e.g. "creation", "amendment", "archival", "disclosure").
    pub lifecycle_stage: Option<String>,

    /// Optional short description of the object (avoid embedding full
    /// clinical content here; audit records should reference, not
    /// duplicate, sensitive data where possible).
    pub description: Option<String>,
}

/// Broad category of the object referenced by a participant object
/// entry — distinguishes a person (patient) from data/system objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantObjectType {
    Patient,
    HealthRecord,
    Document,
    SystemObject,
    Other(String),
}