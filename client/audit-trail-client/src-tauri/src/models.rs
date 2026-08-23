use serde::{Deserialize, Serialize};

/// Mirrors `IotaLogMetadata` from the `audit-trail` service
/// (see audit-trail/src/iota_client.rs). Kept in sync manually since
/// the two services don't currently share a crate.
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

/// Mirrors `LogRecordOnChain` from the `audit-trail` service: the
/// on-chain object id plus the metadata payload stored in it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    pub object_id: String,
    pub metadata: LogMetadata,
    /// Optional: populated once the audit-trail endpoint exposes it.
    /// Not required for the client to function.
    #[serde(default)]
    pub tx_digest: Option<String>,
}

/// Expected shape of `GET {base_url}/api/logs`.
///
/// Adjust field names here once the real audit-trail endpoint is
/// implemented, if they differ from this assumption.
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub audit_trail_base_url: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            // Matches audit-trail/.env PORT=3000 default.
            audit_trail_base_url: "http://localhost:3000".to_string(),
        }
    }
}
