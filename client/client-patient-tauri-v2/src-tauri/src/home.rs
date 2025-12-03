use anyhow::{anyhow, Context};
use serde_json::{json, Value};
use tauri::{async_runtime::Mutex, State};
use umbral_pre::decrypt_original;

use crate::{
    current_fn,
    patient_error::PatientError,
    types::{
        AppState, CommandGetMedicalRecordsResponseData, KeyNonce, MedicalData, MedicalMetadata,
        MovePatientMedicalMetadata, ResponseStatus, SuccessResponse,
    },
    utils::{
        aes_decrypt, get_data_ipfs, get_iota_address_from_keys_entry, get_pre_keys_from_keys_entry,
        parse_keys_entry, serde_deserialize_from_base64,
    },
};

use base64::{engine::general_purpose::STANDARD, Engine as _};

#[tauri::command]
pub async fn get_medical_records(
    state: State<'_, Mutex<AppState>>,
) -> Result<SuccessResponse<Vec<CommandGetMedicalRecordsResponseData>>, PatientError> {
    let state = state.lock().await;
    let keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    let patient_iota_address = {
        let patient_iota_address =
            get_iota_address_from_keys_entry(&keys_entry).context(current_fn!())?;

        patient_iota_address
    };

    let medical_records: Vec<MovePatientMedicalMetadata> = state
        .move_call
        .get_medical_records(0, 10, patient_iota_address)
        .await
        .context(current_fn!())?;

    let medical_records = medical_records
        .into_iter()
        .map(|metadata| {
            let medical_metadata: MedicalMetadata =
                serde_deserialize_from_base64(metadata.metadata).context(current_fn!())?;

            Ok(CommandGetMedicalRecordsResponseData {
                cid: medical_metadata.cid,
                index: metadata.index,
                created_at: medical_metadata.created_at,
            })
        })
        .collect::<Result<Vec<CommandGetMedicalRecordsResponseData>, PatientError>>()?;

    Ok(SuccessResponse {
        status: ResponseStatus::Success,
        data: medical_records,
    })
}

#[tauri::command]
pub async fn get_medical_record(
    state: State<'_, Mutex<AppState>>,
    index: u64,
) -> Result<SuccessResponse<Value>, PatientError> {
    let state = state.lock().await;
    let keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    let (patient_iota_address, patient_pre_secret_key) = {
        let pin = state
            .auth_state
            .session_pin
            .clone()
            .ok_or(anyhow!("Session PIN not found"))
            .context(current_fn!())?;
        let patient_iota_address =
            get_iota_address_from_keys_entry(&keys_entry).context(current_fn!())?;
        let (patient_pre_secret_key, _) =
            get_pre_keys_from_keys_entry(&keys_entry, pin).context(current_fn!())?;

        (patient_iota_address, patient_pre_secret_key)
    };

    let medical_metadata = state
        .move_call
        .get_medical_record(index, patient_iota_address)
        .await
        .context(current_fn!())?;

    let medical_metadata: MedicalMetadata =
        serde_deserialize_from_base64(medical_metadata.metadata)?;

    let medical_record_key_nonce = decrypt_original(
        &patient_pre_secret_key,
        &serde_deserialize_from_base64(medical_metadata.capsule).context(current_fn!())?,
        &STANDARD
            .decode(medical_metadata.enc_key_and_nonce)
            .context(current_fn!())?,
    )
    .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;
    let medical_record_key_nonce: KeyNonce =
        serde_json::from_slice(&medical_record_key_nonce).context(current_fn!())?;

    let medical_record_content = get_data_ipfs(medical_metadata.cid)
        .await
        .context(current_fn!())?;
    let medical_record_content = aes_decrypt(
        &STANDARD
            .decode(medical_record_content)
            .context(current_fn!())?,
        &STANDARD
            .decode(medical_record_key_nonce.key)
            .context(current_fn!())?,
        &STANDARD
            .decode(medical_record_key_nonce.nonce)
            .context(current_fn!())?,
    )
    .context(current_fn!())?;
    let medical_record_content: MedicalData =
        serde_json::from_slice(&medical_record_content).context(current_fn!())?;

    let res_data = json!({
        "createdAt": medical_metadata.created_at,
        "medicalData": medical_record_content,
    });

    Ok(SuccessResponse {
        data: res_data,
        status: ResponseStatus::Success,
    })
}
