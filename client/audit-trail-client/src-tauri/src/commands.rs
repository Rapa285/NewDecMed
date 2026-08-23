use tauri::State;
use tauri_plugin_store::StoreExt;

use crate::error::ClientError;
use crate::models::{AppSettings, FetchLogsParams, LogsResponse};
use crate::state::AppState;

const SETTINGS_STORE_FILE: &str = "settings.json";
const SETTINGS_KEY: &str = "app_settings";

/// Fetch a page of log metadata from the audit-trail service.
#[tauri::command]
pub async fn fetch_logs(
    state: State<'_, AppState>,
    params: FetchLogsParams,
) -> Result<LogsResponse, ClientError> {
    let base_url = state.settings.read().await.audit_trail_base_url.clone();
    state.client.fetch_logs(&base_url, params).await
}

/// Read current settings (currently just the audit-trail base URL).
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, ClientError> {
    Ok(state.settings.read().await.clone())
}

/// Persist settings to disk (via tauri-plugin-store) and update
/// in-memory state so subsequent fetches use the new base URL.
#[tauri::command]
pub async fn save_settings(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), ClientError> {
    let store = app
        .store(SETTINGS_STORE_FILE)
        .map_err(|e| ClientError::Store(e.to_string()))?;

    let value = serde_json::to_value(&settings)
        .map_err(|e| ClientError::Store(e.to_string()))?;
    store.set(SETTINGS_KEY, value);
    store.save().map_err(|e| ClientError::Store(e.to_string()))?;

    *state.settings.write().await = settings;
    Ok(())
}
