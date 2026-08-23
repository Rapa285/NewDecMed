use tokio::sync::RwLock;

use crate::api::AuditorClient;
use crate::types::AppSettings;

pub struct AppState {
    pub client: AuditorClient,
    pub settings: RwLock<AppSettings>,
}

impl AppState {
    pub fn new(settings: AppSettings) -> Self {
        Self {
            client: AuditorClient::new(),
            settings: RwLock::new(settings),
        }
    }
}
