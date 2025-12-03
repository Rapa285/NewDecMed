use anyhow::Context;
use tauri::{async_runtime::Mutex, State};

use crate::{
    current_fn,
    hospital_error::HospitalError,
    types::{AppState, ResponseStatus, SuccessResponse},
    utils::parse_keys_entry,
};

#[tauri::command]
pub async fn signout(
    state: State<'_, Mutex<AppState>>,
) -> Result<SuccessResponse<()>, HospitalError> {
    let state = state.lock().await;

    let mut keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;
    keys_entry.iota_address = None;
    keys_entry.iota_key_pair = None;
    keys_entry.iota_nonce = None;
    keys_entry.pre_nonce = None;
    keys_entry.pre_secret_key = None;
    let keys_entry = serde_json::to_vec(&keys_entry).context(current_fn!())?;
    state
        .keys_entry
        .set_secret(&keys_entry)
        .context(current_fn!())?;

    Ok(SuccessResponse {
        status: ResponseStatus::Success,
        data: (),
    })
}

#[tauri::command]
pub async fn reset(
    state: State<'_, Mutex<AppState>>,
) -> Result<SuccessResponse<()>, HospitalError> {
    let mut state = state.lock().await;

    let mut keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;
    keys_entry.activation_key = None;
    keys_entry.iota_address = None;
    keys_entry.iota_key_pair = None;
    keys_entry.iota_nonce = None;
    keys_entry.pre_nonce = None;
    keys_entry.pre_secret_key = None;
    keys_entry.id = None;
    let keys_entry = serde_json::to_vec(&keys_entry).context(current_fn!())?;
    state
        .keys_entry
        .set_secret(&keys_entry)
        .context(current_fn!())?;
    state.auth_state.role = None;
    state.auth_state.is_signed_up = false;

    Ok(SuccessResponse {
        status: ResponseStatus::Success,
        data: (),
    })
}
