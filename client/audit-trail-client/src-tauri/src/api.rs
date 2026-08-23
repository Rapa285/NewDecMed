use crate::error::ClientError;
use crate::models::{FetchLogsParams, LogsResponse};

#[derive(Clone)]
pub struct AuditTrailClient {
    http: reqwest::Client,
}

impl AuditTrailClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .expect("failed to build reqwest client"),
        }
    }

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

        let parsed = response.json::<LogsResponse>().await?;
        Ok(parsed)
    }
}

impl Default for AuditTrailClient {
    fn default() -> Self {
        Self::new()
    }
}
