use serde::{Deserialize, Serialize};

/// Mirrors `IotaLogMetadata` from the `audit-trail` service
/// (see audit-trail/src/iota_client.rs). Kept in sync manually since
/// the auditor client doesn't share a crate with the backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMetadata {
    pub version: String,
    pub log_sequence_number: u64,
    pub rotation_timestamp: chrono::DateTime<chrono::Utc>,
    pub ipfs_cid: String,
    pub file_hash: String,
    pub first_record_hash: String,
    pub final_record_hash: String,
    pub record_count: u64,
    pub prev_tx_digest: Option<String>,
}

/// Mirrors `LogRecordOnChain`: the on-chain object id plus the
/// metadata payload published for a rotated log batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    pub object_id: String,
    pub metadata: LogMetadata,
}

/// Expected shape of `GET {base_url}/api/logs` on the audit-trail service.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogsResponse {
    pub data: Vec<LogRecord>,
    #[serde(default)]
    pub next_cursor: Option<String>,
    #[serde(default)]
    pub has_next_page: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchLogsParams {
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}

/// A single line inside a rotated audit-trail log file, once
/// downloaded from IPFS and parsed. Mirrors `AuditRecord` /
/// `AuditEvent` from `audit-trail/src/types.rs`, kept loose (Value)
/// here since the auditor only needs to display it, not act on it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub record_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub prev_record_hash: Option<String>,
    pub record_hash: String,
    #[serde(flatten)]
    pub event: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub audit_trail_base_url: String,
    pub ipfs_gateway_base_url: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            // Matches audit-trail/.env PORT=3000 default.
            audit_trail_base_url: "http://localhost:3000".to_string(),
            ipfs_gateway_base_url: "http://103.107.4.68:8080".to_string(),
        }
    }
}
