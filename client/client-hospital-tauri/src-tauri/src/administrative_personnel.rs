use anyhow::{anyhow, Context};
use serde_json::{json, Value};
use tauri::{async_runtime::Mutex, http::StatusCode, State};
use tauri_plugin_http::reqwest;
use umbral_pre::{decrypt_original, decrypt_reencrypted, Capsule, CapsuleFrag, PublicKey};

use crate::{
    constants::PROXY_BASE_URL,
    current_fn,
    hospital_error::HospitalError,
    types::{
        AccessData, AccessMetadata, AccessMetadataEncrypted, AppState, KeyNonce,
        PatientPrivateAdministrativeData, ProxyReencryptionErrorResponse,
        ProxyReencryptionGetPatientAdministrativeDataResponseData,
        ProxyReencryptionSuccessResponse, ResponseStatus, SuccessResponse,
    },
    utils::{
        aes_decrypt, compute_pre_keys, do_http_get_request_json,
        encode_activation_key_from_keys_entry, get_iota_address_from_keys_entry,
        get_iota_key_pair_from_keys_entry, get_pre_keys_from_keys_entry, parse_keys_entry,
        serde_deserialize_from_base64,
    },
};
use base64::{engine::general_purpose::STANDARD, Engine as _};

#[tauri::command]
pub async fn get_administrative_data(
    state: State<'_, Mutex<AppState>>,
    access_token: String,
    patient_iota_address: String,
) -> Result<SuccessResponse<Value>, HospitalError> {
    let state = state.lock().await;
    let keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;
    let req_client = reqwest::Client::new();

    let hospital_personnel_pre_secret_key = {
        let pin = state
            .auth_state
            .session_pin
            .clone()
            .ok_or(anyhow!("Session PIN not found"))?;
        let (hospital_personnel_pre_secret_key, _) =
            get_pre_keys_from_keys_entry(&keys_entry, pin).context(current_fn!())?;

        hospital_personnel_pre_secret_key
    };

    let res = do_http_get_request_json::<
        ProxyReencryptionSuccessResponse<ProxyReencryptionGetPatientAdministrativeDataResponseData>,
        ProxyReencryptionErrorResponse,
        _,
    >(
        Some(access_token),
        &req_client,
        StatusCode::OK,
        format!(
            "{}/administrative?patient_iota_address={}",
            PROXY_BASE_URL, patient_iota_address
        ),
    )
    .await
    .context(current_fn!())?;

    let administrative_data = {
        let patient_pre_public_key: PublicKey =
            serde_deserialize_from_base64(res.data.patient_pre_public_key)
                .context(current_fn!())?;
        let data_pre_secret_key_seed_capsule: Capsule =
            serde_deserialize_from_base64(res.data.data_pre_secret_key_seed_capsule)
                .context(current_fn!())?;
        let data_pre_secret_key_seed = decrypt_original(
            &hospital_personnel_pre_secret_key,
            &data_pre_secret_key_seed_capsule,
            STANDARD
                .decode(res.data.enc_data_pre_secret_key_seed)
                .context(current_fn!())?,
        )
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;
        let (data_pre_secret_key, data_pre_public_key) =
            compute_pre_keys(&data_pre_secret_key_seed).context(current_fn!())?;
        let signer_pre_public_key: PublicKey =
            serde_deserialize_from_base64(res.data.signer_pre_public_key).context(current_fn!())?;
        let c_frag: CapsuleFrag =
            serde_deserialize_from_base64(res.data.c_frag).context(current_fn!())?;
        let administrative_data_capsule: Capsule =
            serde_deserialize_from_base64(res.data.patient_private_adm_data_capsule)
                .context(current_fn!())?;
        let verified_cfrag = c_frag
            .verify(
                &administrative_data_capsule,
                &signer_pre_public_key,
                &patient_pre_public_key,
                &data_pre_public_key,
            )
            .map_err(|e| anyhow!(e.0.to_string()).context(current_fn!()))?;
        let administrative_data_key_nonce = decrypt_reencrypted(
            &data_pre_secret_key,
            &patient_pre_public_key,
            &administrative_data_capsule,
            [verified_cfrag],
            STANDARD
                .decode(res.data.enc_patient_private_adm_data_key_nonce)
                .context(current_fn!())?,
        )
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;
        let administrative_data_key_nonce: KeyNonce =
            serde_json::from_slice(&administrative_data_key_nonce).context(current_fn!())?;
        let administrative_data = aes_decrypt(
            &STANDARD
                .decode(res.data.enc_patient_private_adm_data)
                .context(current_fn!())?,
            &STANDARD
                .decode(administrative_data_key_nonce.key)
                .context(current_fn!())?,
            &STANDARD
                .decode(administrative_data_key_nonce.nonce)
                .context(current_fn!())?,
        )
        .context(current_fn!())?;
        let administrative_data: PatientPrivateAdministrativeData =
            serde_json::from_slice(&administrative_data).context(current_fn!())?;

        administrative_data
    };

    let res_data = json!({
        "administrativeData": administrative_data
    });

    Ok(SuccessResponse {
        data: res_data,
        status: ResponseStatus::Success,
    })
}

#[tauri::command]
pub async fn get_read_access_administrative_personnel(
    state: State<'_, Mutex<AppState>>,
) -> Result<SuccessResponse<Vec<AccessData>>, HospitalError> {
    let state = state.lock().await;
    let keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    let (
        activation_key,
        administrative_personnel_iota_address,
        administrative_personnel_iota_key_pair,
        administrative_personnel_pre_secret_key,
    ) = {
        let pin = state
            .auth_state
            .session_pin
            .clone()
            .ok_or(anyhow!("Session PIN not found on auth state").context(current_fn!()))?;
        let activation_key =
            encode_activation_key_from_keys_entry(&keys_entry).context(current_fn!())?;
        let administrative_personnel_iota_address =
            get_iota_address_from_keys_entry(&keys_entry).context(current_fn!())?;
        let administrative_personnel_iota_key_pair =
            get_iota_key_pair_from_keys_entry(&keys_entry, pin.clone()).context(current_fn!())?;
        let (administrative_personnel_pre_secret_key, _) =
            get_pre_keys_from_keys_entry(&keys_entry, pin).context(current_fn!())?;

        (
            activation_key,
            administrative_personnel_iota_address,
            administrative_personnel_iota_key_pair,
            administrative_personnel_pre_secret_key,
        )
    };

    // do cleanup
    let _ = state
        .move_call
        .cleanup_read_access(
            activation_key.clone(),
            administrative_personnel_iota_address,
            administrative_personnel_iota_key_pair,
        )
        .await
        .context(current_fn!())?;

    // get the data
    let access = state
        .move_call
        .get_read_access(activation_key, administrative_personnel_iota_address)
        .await
        .context(current_fn!())?;

    let access = access
        .into_iter()
        .map(|access| {
            let access_metadata: AccessMetadataEncrypted =
                serde_deserialize_from_base64(access.metadata).context(current_fn!())?;
            let access_metadata = decrypt_original(
                &administrative_personnel_pre_secret_key,
                &serde_deserialize_from_base64(access_metadata.capsule).context(current_fn!())?,
                &STANDARD
                    .decode(access_metadata.enc_data)
                    .context(current_fn!())?,
            )
            .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;
            let access_metadata: AccessMetadata =
                serde_json::from_slice(&access_metadata).context(current_fn!())?;

            let access = AccessData {
                access_data_types: access.access_data_types,
                access_token: access_metadata.access_token,
                exp: access.exp,
                medical_metadata_index: access.medical_metadata_index,
                patient_iota_address: access_metadata.patient_iota_address,
                patient_name: access_metadata.patient_name,
                patient_pre_public_key: access_metadata.patient_pre_public_key,
            };

            Ok(access)
        })
        .collect::<Result<Vec<AccessData>, HospitalError>>()?;

    Ok(SuccessResponse {
        data: access,
        status: ResponseStatus::Success,
    })
}
