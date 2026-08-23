use crate::error::ClientError;
use crate::types::{AuditLogEntry, FetchLogsParams, LogsResponse};

#[derive(Clone)]
pub struct AuditorClient {
    http: reqwest::Client,
}

impl AuditorClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(20))
                .build()
                .expect("failed to build reqwest client"),
        }
    }

    /// `GET {base_url}/api/logs` — page of on-chain `LogRecord` metadata.
    pub async fn fetch_logs(
        &self,
        base_url: &str,
        params: FetchLogsParams,
    ) -> Result<LogsResponse, ClientError> {
        let mut url = reqwest::Url::parse(base_url.trim_end_matches('/'))
            .map_err(|e| ClientError::InvalidUrl(e.to_string()))?;
        url.set_path(&format!("{}/api/logs", url.path().trim_end_matches('/')));

        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(cursor) = params.cursor {
            query.push(("cursor", cursor));
        }
        if let Some(limit) = params.limit {
            query.push(("limit", limit.to_string()));
        }

        let response = self.http.get(url).query(&query).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ClientError::ServerError { status, body });
        }

        Ok(response.json::<LogsResponse>().await?)
    }

    /// Download and parse a rotated log file (newline-delimited JSON
    /// `AuditRecord`s) directly from IPFS via the configured gateway.
    pub async fn fetch_log_file(
        &self,
        ipfs_gateway_base_url: &str,
        cid: &str,
    ) -> Result<Vec<AuditLogEntry>, ClientError> {
        let url = format!(
            "{}/ipfs/{}",
            ipfs_gateway_base_url.trim_end_matches('/'),
            cid
        );

        let response = self.http.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ClientError::ServerError { status, body });
        }

        let body = response.text().await?;

        body.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str::<AuditLogEntry>(line)
                    .map_err(|e| ClientError::Parse(format!("{e}: {line}")))
            })
            .collect()
    }
}

impl Default for AuditorClient {
    fn default() -> Self {
        Self::new()
    }
}
