use anyhow::{anyhow, Context};
use serde_json::{json, Value};
use tauri::{async_runtime::Mutex, State};
use umbral_pre::decrypt_original;

use crate::{
    client_error::ClientError,
    current_fn,
    types::{
        AppState, CommandGetHospitalsPayload, HospitalAdminMetadata,
        HospitalAdminMetadataEncrypted, ResponseStatus, SuccessResponse,
    },
    utils::{
        get_global_admin_iota_address_from_keys_entry, get_global_admin_pre_keys_from_keys_entry,
        parse_keys_entry, serde_deserialize_from_base64,
    },
};

use base64::{engine::general_purpose::STANDARD, Engine as _};

#[tauri::command]
pub async fn get_hospitals(
    state: State<'_, Mutex<AppState>>,
    payload: CommandGetHospitalsPayload,
) -> Result<SuccessResponse<Vec<Value>>, ClientError> {
    let state = state.lock().await;
    let keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    let (admin_iota_address, admin_pre_secret_key) = {
        let admin_iota_address =
            get_global_admin_iota_address_from_keys_entry(&keys_entry).context(current_fn!())?;
        let (admin_pre_secret_key, _) =
            get_global_admin_pre_keys_from_keys_entry(&keys_entry).context(current_fn!())?;

        (admin_iota_address, admin_pre_secret_key)
    };

    let hospitals = state
        .move_call
        .get_hospitals(
            payload.cursor.unwrap_or(0),
            payload.size.unwrap_or(10),
            admin_iota_address,
        )
        .await
        .context(current_fn!())?;

    let hospitals = hospitals
        .into_iter()
        .map(|hospital| {
            let admin_metadata: HospitalAdminMetadataEncrypted =
                serde_deserialize_from_base64(hospital.admin_metadata).context(current_fn!())?;
            let admin_metadata = decrypt_original(
                &admin_pre_secret_key,
                &serde_deserialize_from_base64(admin_metadata.capsule).context(current_fn!())?,
                &STANDARD
                    .decode(admin_metadata.enc_metadata)
                    .context(current_fn!())?,
            )
            .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;
            let admin_metadata: HospitalAdminMetadata =
                serde_json::from_slice(&admin_metadata).context(current_fn!())?;

            Ok(json!({
                "activationKey": admin_metadata.activation_key,
                "hospitalAdminCid": admin_metadata.hospital_admin_cid,
                "hospitalName": hospital.hospital_metadata.name,
            }))
        })
        .collect::<Result<Vec<Value>, ClientError>>()?;

    Ok(SuccessResponse {
        data: hospitals,
        status: ResponseStatus::Success,
    })
}
