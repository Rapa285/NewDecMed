mod api;
mod commands;
mod error;
mod state;
mod types;

use tauri::Manager;
use tauri_plugin_store::StoreExt;

use state::AppState;
use types::AppSettings;

const SETTINGS_STORE_FILE: &str = "settings.json";
const SETTINGS_KEY: &str = "app_settings";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // Load persisted settings if present, otherwise fall back
            // to defaults (see AppSettings::default in types.rs).
            let settings = app
                .store(SETTINGS_STORE_FILE)
                .ok()
                .and_then(|store| store.get(SETTINGS_KEY))
                .and_then(|value| serde_json::from_value::<AppSettings>(value).ok())
                .unwrap_or_default();

            app.manage(AppState::new(settings));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::fetch_logs,
            commands::fetch_log_entries,
            commands::get_settings,
            commands::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
