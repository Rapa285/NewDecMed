use anyhow::Context;
use tauri::{async_runtime::Mutex, State};

use crate::{
    current_fn,
    patient_error::PatientError,
    types::{AppState, ResponseStatus, SuccessResponse},
    utils::parse_keys_entry,
};

#[tauri::command]
pub async fn signout(
    state: State<'_, Mutex<AppState>>,
) -> Result<SuccessResponse<()>, PatientError> {
    let state = state.lock().await;

    let mut keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;
    keys_entry.iota_address = None;
    keys_entry.iota_key_pair = None;
    keys_entry.iota_nonce = None;
    keys_entry.pre_nonce = None;
    keys_entry.pre_secret_key = None;
    keys_entry.pre_public_key = None;
    let keys_entry = serde_json::to_vec(&keys_entry).context(current_fn!())?;
    state
        .keys_entry
        .set_secret(&keys_entry)
        .context(current_fn!())?;

    Ok(SuccessResponse {
        data: (),
        status: ResponseStatus::Success,
    })
}
