use std::str::FromStr;

use anyhow::{anyhow, Context};
use iota_types::base_types::IotaAddress;
use serde_json::{json, Value};
use tauri::{async_runtime::Mutex, http::StatusCode, State};
use tauri_plugin_http::reqwest;
use umbral_pre::{decrypt_original, decrypt_reencrypted, encrypt, Capsule, CapsuleFrag, PublicKey};

use crate::{
    constants::PROXY_BASE_URL,
    current_fn,
    hospital_error::HospitalError,
    types::{
        AccessData, AccessMetadata, AccessMetadataEncrypted, AppState,
        CommandNewMedicalRecordPayload, CommandUpdateMedicalRecordPayload, KeyNonce, MedicalData,
        MedicalMetadata, PatientPrivateAdministrativeData, ProxyReencryptionErrorResponse,
        ProxyReencryptionGetMedicalRecordResponseData,
        ProxyReencryptionGetMedicalRecordUpdateResponseData, ProxyReencryptionSuccessResponse,
        ResponseStatus, SuccessResponse,
    },
    utils::{
        aes_decrypt, aes_encrypt, compute_pre_keys, do_http_get_request_json,
        do_http_post_request_json, do_http_put_request_json, encode_activation_key_from_keys_entry,
        get_iota_address_from_keys_entry, get_iota_key_pair_from_keys_entry,
        get_pre_keys_from_keys_entry, parse_keys_entry, serde_deserialize_from_base64,
        serde_serialize_to_base64,
    },
};
use base64::{engine::general_purpose::STANDARD, Engine as _};

#[tauri::command]
pub async fn new_medical_record(
    _state: State<'_, Mutex<AppState>>,
    access_token: String,
    data: CommandNewMedicalRecordPayload,
    patient_iota_address: String,
    patient_pre_public_key: String,
) -> Result<SuccessResponse<()>, HospitalError> {
    let req_client = reqwest::Client::new();

    let (medical_metadata, patient_iota_address) = {
        let patient_iota_address =
            IotaAddress::from_str(&patient_iota_address).context(current_fn!())?;
        let patient_pre_public_key: PublicKey =
            serde_deserialize_from_base64(patient_pre_public_key).context(current_fn!())?;

        let medical_data = MedicalData {
            anamnesis: data.anamnesis,
            diagnose: data.diagnose,
            physical_check: data.physical_check,
            psychological_check: data.psychological_check,
            therapy: data.therapy,
        };
        let (enc_medical_data, medical_data_key, medical_data_nonce) =
            aes_encrypt(&serde_json::to_vec(&medical_data).context(current_fn!())?)
                .context(current_fn!())?;

        let medical_data_key_nonce = KeyNonce {
            key: STANDARD.encode(medical_data_key),
            nonce: STANDARD.encode(medical_data_nonce),
        };
        let (medical_data_key_nonce_capsule, enc_medical_data_key_nonce) = encrypt(
            &patient_pre_public_key,
            &serde_json::to_vec(&medical_data_key_nonce).context(current_fn!())?,
        )
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;

        let medical_metadata = MedicalMetadata {
            capsule: serde_serialize_to_base64(&medical_data_key_nonce_capsule)
                .context(current_fn!())?,
            enc_data: STANDARD.encode(enc_medical_data),
            enc_key_and_nonce: STANDARD.encode(enc_medical_data_key_nonce),
        };

        (medical_metadata, patient_iota_address)
    };

    let _ = do_http_post_request_json::<
        _,
        ProxyReencryptionSuccessResponse<()>,
        ProxyReencryptionErrorResponse,
    >(
        Some(access_token),
        &format!("{}/medical-record", PROXY_BASE_URL),
        &json!({
            "medical_metadata": serde_serialize_to_base64(&medical_metadata).context(current_fn!())?,
            "patient_iota_address": patient_iota_address.to_string(),
        }),
        &req_client,
        StatusCode::OK,
    )
    .await
    .context(current_fn!())?;

    Ok(SuccessResponse {
        status: ResponseStatus::Success,
        data: (),
    })
}

#[tauri::command]
pub async fn get_medical_record(
    state: State<'_, Mutex<AppState>>,
    access_token: String,
    index: Option<u64>,
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
        ProxyReencryptionSuccessResponse<ProxyReencryptionGetMedicalRecordResponseData>,
        ProxyReencryptionErrorResponse,
        _,
    >(
        Some(access_token),
        &req_client,
        StatusCode::OK,
        format!(
            "{}/medical-record?index={}&patient_iota_address={}",
            PROXY_BASE_URL,
            index.unwrap_or(0),
            patient_iota_address
        ),
    )
    .await
    .context(current_fn!())?;

    let (medical_data, administrative_data) = {
        let patient_pre_public_key: PublicKey =
            serde_deserialize_from_base64(res.data.patient_pre_public_key)
                .context(current_fn!())?;
        let medical_record_pre_secret_key_seed_capsule: Capsule =
            serde_deserialize_from_base64(res.data.data_pre_secret_key_seed_capsule)
                .context(current_fn!())?;
        let medical_record_pre_secret_key_seed = decrypt_original(
            &hospital_personnel_pre_secret_key,
            &medical_record_pre_secret_key_seed_capsule,
            STANDARD
                .decode(res.data.enc_data_pre_secret_key_seed)
                .context(current_fn!())?,
        )
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;
        let (medical_record_pre_secret_key, medical_record_pre_public_key) =
            compute_pre_keys(&medical_record_pre_secret_key_seed).context(current_fn!())?;
        let signer_pre_public_key: PublicKey =
            serde_deserialize_from_base64(res.data.signer_pre_public_key).context(current_fn!())?;
        let c_frag_medical: CapsuleFrag =
            serde_deserialize_from_base64(res.data.c_frag_medical).context(current_fn!())?;
        let medical_data_capsule: Capsule =
            serde_deserialize_from_base64(res.data.medical_data_capsule).context(current_fn!())?;
        let verified_cfrag_medical = c_frag_medical
            .verify(
                &medical_data_capsule,
                &signer_pre_public_key,
                &patient_pre_public_key,
                &medical_record_pre_public_key,
            )
            .map_err(|e| anyhow!(e.0.to_string()).context(current_fn!()))?;
        let medical_data_key_nonce = decrypt_reencrypted(
            &medical_record_pre_secret_key,
            &patient_pre_public_key,
            &medical_data_capsule,
            [verified_cfrag_medical],
            STANDARD
                .decode(res.data.enc_medical_data_key_nonce)
                .context(current_fn!())?,
        )
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;
        let medical_data_key_nonce: KeyNonce =
            serde_json::from_slice(&medical_data_key_nonce).context(current_fn!())?;
        let medical_data = aes_decrypt(
            &STANDARD
                .decode(res.data.enc_medical_data)
                .context(current_fn!())?,
            &STANDARD
                .decode(medical_data_key_nonce.key)
                .context(current_fn!())?,
            &STANDARD
                .decode(medical_data_key_nonce.nonce)
                .context(current_fn!())?,
        )
        .context(current_fn!())?;
        let medical_data: MedicalData =
            serde_json::from_slice(&medical_data).context(current_fn!())?;

        let c_frag_administrative: CapsuleFrag =
            serde_deserialize_from_base64(res.data.c_frag_administrative).context(current_fn!())?;
        let administrative_data_capsule: Capsule =
            serde_deserialize_from_base64(res.data.administrative_data_capsule)
                .context(current_fn!())?;
        let verified_cfrag_administrative = c_frag_administrative
            .verify(
                &administrative_data_capsule,
                &signer_pre_public_key,
                &patient_pre_public_key,
                &medical_record_pre_public_key,
            )
            .map_err(|e| anyhow!(e.0.to_string()).context(current_fn!()))?;
        let administrative_data_key_nonce = decrypt_reencrypted(
            &medical_record_pre_secret_key,
            &patient_pre_public_key,
            &administrative_data_capsule,
            [verified_cfrag_administrative],
            STANDARD
                .decode(res.data.enc_administrative_data_key_nonce)
                .context(current_fn!())?,
        )
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;
        let administrative_data_key_nonce: KeyNonce =
            serde_json::from_slice(&administrative_data_key_nonce).context(current_fn!())?;
        let administrative_data = aes_decrypt(
            &STANDARD
                .decode(res.data.enc_administrative_data)
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

        (medical_data, administrative_data)
    };

    let res_data = json!({
        "administrativeData": administrative_data,
        "createdAt": res.data.medical_data_created_at,
        "currentIndex": res.data.current_index,
        "medicalData": medical_data,
        "nextIndex": res.data.next_index,
        "prevIndex": res.data.prev_index,
    });

    Ok(SuccessResponse {
        data: res_data,
        status: ResponseStatus::Success,
    })
}

#[tauri::command]
pub async fn get_medical_record_update(
    state: State<'_, Mutex<AppState>>,
    access_token: String,
    index: u64,
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
        ProxyReencryptionSuccessResponse<ProxyReencryptionGetMedicalRecordUpdateResponseData>,
        ProxyReencryptionErrorResponse,
        _,
    >(
        Some(access_token),
        &req_client,
        StatusCode::OK,
        format!(
            "{}/medical-record-update?index={}&patient_iota_address={}",
            PROXY_BASE_URL, index, patient_iota_address
        ),
    )
    .await
    .context(current_fn!())?;

    let (medical_data, administrative_data) = {
        let patient_pre_public_key: PublicKey =
            serde_deserialize_from_base64(res.data.patient_pre_public_key)
                .context(current_fn!())?;
        let medical_record_pre_secret_key_seed_capsule: Capsule =
            serde_deserialize_from_base64(res.data.data_pre_secret_key_seed_capsule)
                .context(current_fn!())?;
        let medical_record_pre_secret_key_seed = decrypt_original(
            &hospital_personnel_pre_secret_key,
            &medical_record_pre_secret_key_seed_capsule,
            STANDARD
                .decode(res.data.enc_data_pre_secret_key_seed)
                .context(current_fn!())?,
        )
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;
        let (medical_record_pre_secret_key, medical_record_pre_public_key) =
            compute_pre_keys(&medical_record_pre_secret_key_seed).context(current_fn!())?;
        let signer_pre_public_key: PublicKey =
            serde_deserialize_from_base64(res.data.signer_pre_public_key).context(current_fn!())?;
        let c_frag_medical: CapsuleFrag =
            serde_deserialize_from_base64(res.data.c_frag_medical).context(current_fn!())?;
        let medical_data_capsule: Capsule =
            serde_deserialize_from_base64(res.data.medical_data_capsule).context(current_fn!())?;
        let verified_cfrag_medical = c_frag_medical
            .verify(
                &medical_data_capsule,
                &signer_pre_public_key,
                &patient_pre_public_key,
                &medical_record_pre_public_key,
            )
            .map_err(|e| anyhow!(e.0.to_string()).context(current_fn!()))?;
        let medical_data_key_nonce = decrypt_reencrypted(
            &medical_record_pre_secret_key,
            &patient_pre_public_key,
            &medical_data_capsule,
            [verified_cfrag_medical],
            STANDARD
                .decode(res.data.enc_medical_data_key_nonce)
                .context(current_fn!())?,
        )
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;
        let medical_data_key_nonce: KeyNonce =
            serde_json::from_slice(&medical_data_key_nonce).context(current_fn!())?;
        let medical_data = aes_decrypt(
            &STANDARD
                .decode(res.data.enc_medical_data)
                .context(current_fn!())?,
            &STANDARD
                .decode(medical_data_key_nonce.key)
                .context(current_fn!())?,
            &STANDARD
                .decode(medical_data_key_nonce.nonce)
                .context(current_fn!())?,
        )
        .context(current_fn!())?;
        let medical_data: MedicalData =
            serde_json::from_slice(&medical_data).context(current_fn!())?;

        let c_frag_administrative: CapsuleFrag =
            serde_deserialize_from_base64(res.data.c_frag_administrative).context(current_fn!())?;
        let administrative_data_capsule: Capsule =
            serde_deserialize_from_base64(res.data.administrative_data_capsule)
                .context(current_fn!())?;
        let verified_cfrag_administrative = c_frag_administrative
            .verify(
                &administrative_data_capsule,
                &signer_pre_public_key,
                &patient_pre_public_key,
                &medical_record_pre_public_key,
            )
            .map_err(|e| anyhow!(e.0.to_string()).context(current_fn!()))?;
        let administrative_data_key_nonce = decrypt_reencrypted(
            &medical_record_pre_secret_key,
            &patient_pre_public_key,
            &administrative_data_capsule,
            [verified_cfrag_administrative],
            STANDARD
                .decode(res.data.enc_administrative_data_key_nonce)
                .context(current_fn!())?,
        )
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;
        let administrative_data_key_nonce: KeyNonce =
            serde_json::from_slice(&administrative_data_key_nonce).context(current_fn!())?;
        let administrative_data = aes_decrypt(
            &STANDARD
                .decode(res.data.enc_administrative_data)
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

        (medical_data, administrative_data)
    };

    let res_data = json!({
        "administrativeData": administrative_data,
        "createdAt": res.data.medical_data_created_at,
        "medicalData": medical_data,
    });

    Ok(SuccessResponse {
        data: res_data,
        status: ResponseStatus::Success,
    })
}

#[tauri::command]
pub async fn get_read_access_medical_personnel(
    state: State<'_, Mutex<AppState>>,
) -> Result<SuccessResponse<Vec<AccessData>>, HospitalError> {
    let state = state.lock().await;
    let keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    let (
        activation_key,
        medical_personnel_iota_address,
        medical_personnel_iota_key_pair,
        medical_personnel_pre_secret_key,
    ) = {
        let pin = state
            .auth_state
            .session_pin
            .clone()
            .ok_or(anyhow!("Session PIN not found on auth state").context(current_fn!()))?;
        let activation_key =
            encode_activation_key_from_keys_entry(&keys_entry).context(current_fn!())?;
        let medical_personnel_iota_address =
            get_iota_address_from_keys_entry(&keys_entry).context(current_fn!())?;
        let medical_personnel_iota_key_pair =
            get_iota_key_pair_from_keys_entry(&keys_entry, pin.clone()).context(current_fn!())?;
        let (medical_personnel_pre_secret_key, _) =
            get_pre_keys_from_keys_entry(&keys_entry, pin).context(current_fn!())?;

        (
            activation_key,
            medical_personnel_iota_address,
            medical_personnel_iota_key_pair,
            medical_personnel_pre_secret_key,
        )
    };

    // do cleanup
    let _ = state
        .move_call
        .cleanup_read_access(
            activation_key.clone(),
            medical_personnel_iota_address,
            medical_personnel_iota_key_pair,
        )
        .await
        .context(current_fn!())?;

    // get the data
    let access = state
        .move_call
        .get_read_access(activation_key, medical_personnel_iota_address)
        .await
        .context(current_fn!())?;

    let access = access
        .into_iter()
        .map(|access| {
            let access_metadata: AccessMetadataEncrypted =
                serde_deserialize_from_base64(access.metadata).context(current_fn!())?;
            let access_metadata = decrypt_original(
                &medical_personnel_pre_secret_key,
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

#[tauri::command]
pub async fn get_update_access_medical_personnel(
    state: State<'_, Mutex<AppState>>,
) -> Result<SuccessResponse<Vec<AccessData>>, HospitalError> {
    let state = state.lock().await;
    let keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    let (
        activation_key,
        medical_personnel_iota_address,
        medical_personnel_iota_key_pair,
        medical_personnel_pre_secret_key,
    ) = {
        let pin = state
            .auth_state
            .session_pin
            .clone()
            .ok_or(anyhow!("Session PIN not found on auth state").context(current_fn!()))?;
        let activation_key =
            encode_activation_key_from_keys_entry(&keys_entry).context(current_fn!())?;
        let medical_personnel_iota_address =
            get_iota_address_from_keys_entry(&keys_entry).context(current_fn!())?;
        let medical_personnel_iota_key_pair =
            get_iota_key_pair_from_keys_entry(&keys_entry, pin.clone()).context(current_fn!())?;
        let (medical_personnel_pre_secret_key, _) =
            get_pre_keys_from_keys_entry(&keys_entry, pin).context(current_fn!())?;

        (
            activation_key,
            medical_personnel_iota_address,
            medical_personnel_iota_key_pair,
            medical_personnel_pre_secret_key,
        )
    };

    // do cleanup
    let _ = state
        .move_call
        .cleanup_update_access(
            activation_key.clone(),
            medical_personnel_iota_address,
            medical_personnel_iota_key_pair,
        )
        .await
        .context(current_fn!())?;

    // get the data
    let access = state
        .move_call
        .get_update_access(activation_key, medical_personnel_iota_address)
        .await
        .context(current_fn!())?;

    let access = access
        .into_iter()
        .map(|access| {
            let access_metadata: AccessMetadataEncrypted =
                serde_deserialize_from_base64(access.metadata).context(current_fn!())?;
            let access_metadata = decrypt_original(
                &medical_personnel_pre_secret_key,
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

#[tauri::command]
pub async fn update_medical_record(
    _state: State<'_, Mutex<AppState>>,
    access_token: String,
    data: CommandUpdateMedicalRecordPayload,
    patient_iota_address: String,
    patient_pre_public_key: String,
) -> Result<SuccessResponse<()>, HospitalError> {
    let req_client = reqwest::Client::new();

    let (medical_metadata, patient_iota_address) = {
        let patient_iota_address =
            IotaAddress::from_str(&patient_iota_address).context(current_fn!())?;
        let patient_pre_public_key: PublicKey =
            serde_deserialize_from_base64(patient_pre_public_key).context(current_fn!())?;

        let medical_data = MedicalData {
            anamnesis: data.anamnesis,
            diagnose: data.diagnose,
            physical_check: data.physical_check,
            psychological_check: data.psychological_check,
            therapy: data.therapy,
        };
        let (enc_medical_data, medical_data_key, medical_data_nonce) =
            aes_encrypt(&serde_json::to_vec(&medical_data).context(current_fn!())?)
                .context(current_fn!())?;

        let medical_data_key_nonce = KeyNonce {
            key: STANDARD.encode(medical_data_key),
            nonce: STANDARD.encode(medical_data_nonce),
        };
        let (medical_data_key_nonce_capsule, enc_medical_data_key_nonce) = encrypt(
            &patient_pre_public_key,
            &serde_json::to_vec(&medical_data_key_nonce).context(current_fn!())?,
        )
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;

        let medical_metadata = MedicalMetadata {
            capsule: serde_serialize_to_base64(&medical_data_key_nonce_capsule)
                .context(current_fn!())?,
            enc_data: STANDARD.encode(enc_medical_data),
            enc_key_and_nonce: STANDARD.encode(enc_medical_data_key_nonce),
        };

        (medical_metadata, patient_iota_address)
    };

    let _ = do_http_put_request_json::<
        _,
        ProxyReencryptionSuccessResponse<()>,
        ProxyReencryptionErrorResponse,
    >(
        Some(access_token),
        &format!("{}/medical-record", PROXY_BASE_URL),
        &json!({
            "medical_metadata": serde_serialize_to_base64(&medical_metadata).context(current_fn!())?,
            "patient_iota_address": patient_iota_address.to_string(),
        }),
        &req_client,
        StatusCode::OK,
    )
    .await
    .context(current_fn!())?;

    Ok(SuccessResponse {
        status: ResponseStatus::Success,
        data: (),
    })
}
