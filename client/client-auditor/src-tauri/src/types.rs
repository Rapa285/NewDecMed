use serde::{Deserialize, Serialize};

/// Matches the AuditEvent structure stored in ATS (as JSON in on-chain `json_data`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub source_component: Option<String>,
    pub actor: Option<String>,
    pub target_object: Option<String>,
    pub outcome: Option<String>,
    pub action_type: Option<String>,
    pub details: Option<serde_json::Value>,
    /// Timestamp injected by ATS before storing
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Sequential index in on-chain store
    #[serde(default)]
    pub sequence: Option<u64>,
}

/// A page of raw on-chain log records returned by the ATS metadata endpoint.
/// The ATS HTTP service wraps each `LogRecord.json_data` string here.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogsMetadataResponse {
    /// Parsed audit log entries
    pub data: Vec<AuditLogEntry>,
    /// Total count stored on-chain
    pub total: u64,
    /// Cursor used for this page
    pub cursor: u64,
    /// How many were returned
    pub limit: u64,
    /// Whether more records exist after this page
    pub has_next_page: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchLogsParams {
    pub cursor: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub ats_base_url: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            // Matches ATS_BASE_URL from constants.rs in ministry/hospital clients
            ats_base_url: "http://localhost:3000".to_string(),
        }
    }
}