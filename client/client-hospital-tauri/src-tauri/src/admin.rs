use anyhow::{anyhow, Context};
use tauri::{async_runtime::Mutex, State};
use umbral_pre::{decrypt_original, Capsule};

use crate::{
    current_fn,
    hospital_error::HospitalError,
    types::{
        AppState, CommandGetHospitalPersonnelsResponseData, HospitalPersonnelMetadata,
        MoveCallHospitalAdminAddActivationKeyPayload, ResponseStatus, SuccessResponse,
    },
    utils::{
        encode_activation_key_from_keys_entry, get_iota_address_from_keys_entry,
        get_pre_keys_from_keys_entry, parse_keys_entry, serde_deserialize_from_base64,
    },
};
use base64::{engine::general_purpose::STANDARD, Engine as _};

#[tauri::command]
pub async fn get_hospital_personnels(
    state: State<'_, Mutex<AppState>>,
) -> Result<SuccessResponse<CommandGetHospitalPersonnelsResponseData>, HospitalError> {
    let state = state.lock().await;
    let keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    let (hospital_admin_pre_secret_key, hospital_admin_iota_address, activation_key) = {
        let activation_key =
            encode_activation_key_from_keys_entry(&keys_entry).context(current_fn!())?;
        let hospital_admin_iota_address =
            get_iota_address_from_keys_entry(&keys_entry).context(current_fn!())?;
        let (hospital_admin_pre_secret_key, _) = get_pre_keys_from_keys_entry(
            &keys_entry,
            state
                .auth_state
                .session_pin
                .clone()
                .ok_or(anyhow!("Session PIN not found").context(current_fn!()))?,
        )?;

        (
            hospital_admin_pre_secret_key,
            hospital_admin_iota_address,
            activation_key,
        )
    };

    let hospital_personnels_metadata = state
        .move_call
        .get_hospital_personnels(activation_key, hospital_admin_iota_address)
        .await
        .context(current_fn!())?;

    let hospital_personnels_metadata = hospital_personnels_metadata
        .iter()
        .map(|metadata| {
            let metadata: MoveCallHospitalAdminAddActivationKeyPayload = serde_json::from_slice(
                &(STANDARD
                    .decode(metadata.metadata.clone())
                    .context(current_fn!())?),
            )
            .context(current_fn!())?;
            let capsule: Capsule =
                serde_deserialize_from_base64(metadata.capsule).context(current_fn!())?;
            let ori = decrypt_original(
                &hospital_admin_pre_secret_key,
                &capsule,
                &STANDARD
                    .decode(metadata.enc_metadata)
                    .context(current_fn!())?,
            )
            .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;
            Ok(
                serde_json::from_slice::<HospitalPersonnelMetadata>(&*ori)
                    .context(current_fn!())?,
            )
        })
        .collect::<Result<Vec<HospitalPersonnelMetadata>, HospitalError>>()
        .context(current_fn!())?;

    let data = CommandGetHospitalPersonnelsResponseData {
        personnels: hospital_personnels_metadata,
    };

    Ok(SuccessResponse {
        data,
        status: ResponseStatus::Success,
    })
}
