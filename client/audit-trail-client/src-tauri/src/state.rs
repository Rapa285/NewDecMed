use tokio::sync::RwLock;

use crate::api::AuditTrailClient;
use crate::models::AppSettings;

pub struct AppState {
    pub client: AuditTrailClient,
    pub settings: RwLock<AppSettings>,
}

impl AppState {
    pub fn new(settings: AppSettings) -> Self {
        Self {
            client: AuditTrailClient::new(),
            settings: RwLock::new(settings),
        }
    }
}
