use std::str::FromStr;

use anyhow::{anyhow, Context};
use iota_types::base_types::IotaAddress;
use serde_json::{json, Value};
use tauri::{async_runtime::Mutex, State};

use crate::{
    current_fn,
    patient_error::PatientError,
    types::{
        AppState, HospitalPersonnelPublicAdministrativeData, MovePatientAccessLog, ResponseStatus,
        SuccessResponse,
    },
    utils::{
        get_iota_address_from_keys_entry, get_iota_key_pair_from_keys_entry, parse_keys_entry,
        serde_deserialize_from_base64,
    },
};

#[tauri::command]
pub async fn get_access_log(
    state: State<'_, Mutex<AppState>>,
) -> Result<SuccessResponse<Vec<Value>>, PatientError> {
    let state = state.lock().await;
    let keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    let patient_iota_address = {
        let patient_iota_address =
            get_iota_address_from_keys_entry(&keys_entry).context(current_fn!())?;

        patient_iota_address
    };

    let access_log: Vec<MovePatientAccessLog> = state
        .move_call
        .get_access_log(0, 10, patient_iota_address)
        .await
        .context(current_fn!())?;

    let access_log = access_log
        .into_iter()
        .map(|metadata| {
            let hospital_personnel_metadata: HospitalPersonnelPublicAdministrativeData =
                serde_deserialize_from_base64(metadata.hospital_personnel_metadata)
                    .context(current_fn!())?;

            Ok(json!({
                "access_data_type": metadata.access_data_type,
                "access_type": metadata.access_type,
                "date": metadata.date,
                "exp_dur": metadata.exp_dur,
                "hospital_metadata": metadata.hospital_metadata,
                "hospital_personnel_address": metadata.hospital_personnel_address,
                "hospital_personnel_metadata": hospital_personnel_metadata,
                "index": metadata.index,
                "is_revoked": metadata.is_revoked,
            }))
        })
        .collect::<Result<Vec<Value>, PatientError>>()?;

    Ok(SuccessResponse {
        data: access_log,
        status: ResponseStatus::Success,
    })
}

#[tauri::command]
pub async fn revoke_access(
    state: State<'_, Mutex<AppState>>,
    hospital_personnel_address: String,
    index: u64,
) -> Result<SuccessResponse<()>, PatientError> {
    let state = state.lock().await;
    let keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    let (patient_iota_address, patient_iota_key_pair, hospital_personnel_address) = {
        let pin = state
            .auth_state
            .session_pin
            .clone()
            .ok_or(anyhow!("Session PIN Not found"))
            .context(current_fn!())?;
        let patient_iota_address =
            get_iota_address_from_keys_entry(&keys_entry).context(current_fn!())?;
        let patient_iota_key_pair =
            get_iota_key_pair_from_keys_entry(&keys_entry, pin.clone()).context(current_fn!())?;
        let hospital_personnel_address =
            IotaAddress::from_str(&hospital_personnel_address).context(current_fn!())?;

        (
            patient_iota_address,
            patient_iota_key_pair,
            hospital_personnel_address,
        )
    };

    let _ = state
        .move_call
        .revoke_access(
            hospital_personnel_address,
            index,
            patient_iota_address,
            patient_iota_key_pair,
        )
        .await
        .context(current_fn!())?;

    Ok(SuccessResponse {
        data: (),
        status: ResponseStatus::Success,
    })
}
