use serde::{Serialize, Deserialize};

// ── Encrypted transport wrapper ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSignedEvent {
    pub enc_aes_key: String,   // base64: ECIES(ephemeral_pubkey.wrap_nonce.enc_key)
    pub ciphertext: String,    // base64: AuditEvent terenkripsi AES-256-GCM
    pub nonce: String,         // base64: nonce AES-GCM (12 bytes)
    pub signature: String,     // base64: IotaSignature atas ciphertext bytes
    pub iota_address: String,  // untuk verifikasi signature di ATS server
}

// ── Enums (harus cocok dengan audit-trail/src/types.rs) ──────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum AuditOutcome {
    Success,
    Failure,
    Denied,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum AuditSourceComponent {
    HospitalClient,
    PatientClient,
    MinistryClient,
    ProxyReencryption,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditActionType {
    Signin,
    Signup,
    Signout,
    ValidatePin,
    ValidateSeedWords,
    UseActivationKey,
    QrScan,
    QrDecode,
    QrValidate,
    CreateAccess,
    StoreKeys,
    RevokeAccess,
    ReadMedicalRecord,
    CreateMedicalRecord,
    UpdateMedicalRecord,
    ReadAdministrativeData,
    CreatePersonnelActivationKey,
    UpdatePersonnelActivationKey,
    CreateFacilityActivationKey,
    UpdateFacilityActivationKey,
    NonceRequest,
    PreReencrypt,
    JwtIssue,
    AuthMiddlewareCheck,
    IotaTransaction,
    GasReserve,
    GasExecute,
    RedisGet,
    RedisSet,
    RedisDel,
    IpfsUpload,
    IpfsFetch,
    ChainRead,
    ChainWrite,
    JwtValidate,
    RoleCheck,
    PurposeCheck,
    ProxyCapValidate,
    ScopeValidate,
}

// ── AuditEvent ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub source_component: AuditSourceComponent,
    pub actor: String,
    pub target_object: String,
    pub outcome: AuditOutcome,
    pub action_type: AuditActionType,
    pub details: AuditEventDetails,
}

// ── AuditEventDetails (harus cocok dengan audit-trail/src/types.rs) ───────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum AuditEventDetails {
    /// EV7 - PRE Service Request/Response
    #[serde(rename = "EV7")]
    PreServiceOperation {
        endpoint_called: String,
        request_id: String,
        caller_component: String,
        channel_encryption: String,
        jwt_purpose: Option<String>,
        http_status_code: u16,
        latency_ms: u64,
    },

    /// EV8 - IOTA Transaction Submission
    #[serde(rename = "EV8")]
    IotaTransaction {
        transaction_digest: String,
        payload_hash: String,
        move_function: String,
        move_module: String,
        network_confirmation_status: String,
        gas_used: Option<u64>,
        sponsor_address: String,
    },

    /// EV9 - Gas Sponsorship Request
    #[serde(rename = "EV9")]
    GasSponsorship {
        gas_budget_requested: u64,
        reserve_duration_secs: u64,
        reservation_id: Option<u64>,
        sponsor_address: Option<String>,
        transaction_digest: Option<String>,
        gas_coin_object_ids: Vec<String>,
    },

    /// EV10 - Redis Operation
    #[serde(rename = "EV10")]
    RedisOperation {
        redis_key_type: String,
        operation_type: String,
        ttl_remaining: Option<i64>,
        key_pattern: String,
    },

    /// EV11 - IPFS Object Access
    #[serde(rename = "EV11")]
    IpfsOperation {
        cid: String,
        operation_type: String,
        data_size: Option<u64>,
        patient_iota_address: Option<String>,
        ipfs_node_url: String,
    },

    /// EV12 - On-chain Metadata Access
    #[serde(rename = "EV12")]
    IotaMetadataOperation {
        object_id: String,
        object_type: String,
        move_function: String,
        is_mutable: bool,
        transaction_digest: Option<String>,
        dev_inspect_used: bool,
    },

    /// EV13 - Capability Validation
    #[serde(rename = "EV13")]
    CapabilityValidation {
        capability_id: String,
        required_scope: String,
        actual_scope: String,
        rejection_reason: Option<String>,
        middleware_layer: String,
    },
}

impl AuditEventDetails {
    pub fn event_type(&self) -> &'static str {
        match self {
            AuditEventDetails::PreServiceOperation { .. } => "EV7",
            AuditEventDetails::IotaTransaction { .. } => "EV8",
            AuditEventDetails::GasSponsorship { .. } => "EV9",
            AuditEventDetails::RedisOperation { .. } => "EV10",
            AuditEventDetails::IpfsOperation { .. } => "EV11",
            AuditEventDetails::IotaMetadataOperation { .. } => "EV12",
            AuditEventDetails::CapabilityValidation { .. } => "EV13",
        }
    }
}

impl AuditEvent {
    pub fn event_type(&self) -> &'static str {
        self.details.event_type()
    }
}