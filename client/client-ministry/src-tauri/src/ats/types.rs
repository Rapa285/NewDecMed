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
    #[serde(rename = "EV5")]
    FacilityRegistration {
        facility_id: String,
        facility_name: String,
        administrator_id: String,
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
}