use anyhow::{anyhow, Context};
use serde_json::{json, Value};
use tauri::{async_runtime::Mutex, State};
use umbral_pre::encrypt;
use uuid::Uuid;

use crate::{
    client_error::ClientError,
    current_fn,
    types::{
        AppState, CommandCreateActivationKeyPayload, CommandUpdateActivationKeyPayload,
        HospitalAdminMetadata, HospitalAdminMetadataEncrypted, ResponseStatus, SuccessResponse,
    },
    utils::{
        argon_hash, decode_hospital_personnel_id, generate_64_bytes_seed,
        get_global_admin_iota_address_from_keys_entry,
        get_global_admin_iota_key_pair_from_keys_entry, get_global_admin_pre_keys_from_keys_entry,
        parse_keys_entry, serde_serialize_to_base64,
    },
};

use base64::{engine::general_purpose::STANDARD, Engine as _};

#[tauri::command]
pub async fn create_activation_key(
    state: State<'_, Mutex<AppState>>,
    payload: CommandCreateActivationKeyPayload,
) -> Result<SuccessResponse<()>, ClientError> {
    let state = state.lock().await;
    let keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    let (admin_iota_address, admin_iota_key_pair, admin_pre_public_key) = {
        let admin_iota_address =
            get_global_admin_iota_address_from_keys_entry(&keys_entry).context(current_fn!())?;
        let admin_iota_key_pair =
            get_global_admin_iota_key_pair_from_keys_entry(&keys_entry).context(current_fn!())?;
        let (_, admin_pre_public_key) =
            get_global_admin_pre_keys_from_keys_entry(&keys_entry).context(current_fn!())?;

        (
            admin_iota_address,
            admin_iota_key_pair,
            admin_pre_public_key,
        )
    };

    let hospital_admin_id = "admin";
    let hospital_admin_cid = format!("{}@{}", hospital_admin_id, payload.hospital_id);
    let activation_key = Uuid::new_v4().to_string();
    let compound_activation_key = format!("{}@{}", activation_key, hospital_admin_cid);

    let hospital_admin_metadata = HospitalAdminMetadata {
        activation_key: activation_key.clone(),
        hospital_admin_cid: hospital_admin_cid.clone(),
    };

    let (hospital_admin_metadata_capsule, enc_hospital_admin_metadata) = encrypt(
        &admin_pre_public_key,
        &serde_json::to_vec(&hospital_admin_metadata).context(current_fn!())?,
    )
    .map_err(|e| anyhow!(e.to_string()))?;

    let hospital_admin_metadata_enc = HospitalAdminMetadataEncrypted {
        capsule: serde_serialize_to_base64(&hospital_admin_metadata_capsule)
            .context(current_fn!())?,
        enc_metadata: STANDARD.encode(enc_hospital_admin_metadata),
    };

    let _ = state
        .move_call
        .create_activation_key(
            STANDARD.encode(argon_hash(compound_activation_key).context(current_fn!())?),
            argon_hash(hospital_admin_id.to_string()).context(current_fn!())?,
            serde_serialize_to_base64(&hospital_admin_metadata_enc).context(current_fn!())?,
            argon_hash(payload.hospital_id).context(current_fn!())?,
            payload.hospital_name,
            admin_iota_address,
            admin_iota_key_pair,
        )
        .await
        .context(current_fn!())?;

    Ok(SuccessResponse {
        data: (),
        status: ResponseStatus::Success,
    })
}

#[tauri::command]
pub async fn generate_pre_seed() -> Result<SuccessResponse<Value>, ClientError> {
    let pre_seed = generate_64_bytes_seed();

    let res_data = json!({
        "pre_seed": STANDARD.encode(&pre_seed[0..32]),
    });

    Ok(SuccessResponse {
        data: res_data,
        status: ResponseStatus::Success,
    })
}

#[tauri::command]
pub async fn update_activation_key(
    state: State<'_, Mutex<AppState>>,
    payload: CommandUpdateActivationKeyPayload,
) -> Result<SuccessResponse<()>, ClientError> {
    let state = state.lock().await;
    let keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    let (admin_iota_address, admin_iota_key_pair, admin_pre_public_key) = {
        let admin_iota_address =
            get_global_admin_iota_address_from_keys_entry(&keys_entry).context(current_fn!())?;
        let admin_iota_key_pair =
            get_global_admin_iota_key_pair_from_keys_entry(&keys_entry).context(current_fn!())?;
        let (_, admin_pre_public_key) =
            get_global_admin_pre_keys_from_keys_entry(&keys_entry).context(current_fn!())?;

        (
            admin_iota_address,
            admin_iota_key_pair,
            admin_pre_public_key,
        )
    };

    let (admin_id, hospital_id) =
        decode_hospital_personnel_id(payload.hospital_admin_cid.clone()).context(current_fn!())?;
    let activation_key = Uuid::new_v4().to_string();
    let compound_activation_key =
        format!("{}@{}", activation_key, payload.hospital_admin_cid.clone());

    let hospital_admin_metadata = HospitalAdminMetadata {
        activation_key: activation_key.clone(),
        hospital_admin_cid: payload.hospital_admin_cid,
    };

    let (hospital_admin_metadata_capsule, enc_hospital_admin_metadata) = encrypt(
        &admin_pre_public_key,
        &serde_json::to_vec(&hospital_admin_metadata).context(current_fn!())?,
    )
    .map_err(|e| anyhow!(e.to_string()))?;

    let hospital_admin_metadata_enc = HospitalAdminMetadataEncrypted {
        capsule: serde_serialize_to_base64(&hospital_admin_metadata_capsule)
            .context(current_fn!())?,
        enc_metadata: STANDARD.encode(enc_hospital_admin_metadata),
    };

    let _ = state
        .move_call
        .update_activation_key(
            STANDARD.encode(argon_hash(compound_activation_key).context(current_fn!())?),
            argon_hash(admin_id).context(current_fn!())?,
            serde_serialize_to_base64(&hospital_admin_metadata_enc).context(current_fn!())?,
            argon_hash(hospital_id).context(current_fn!())?,
            admin_iota_address,
            admin_iota_key_pair,
        )
        .await
        .context(current_fn!())?;

    Ok(SuccessResponse {
        data: (),
        status: ResponseStatus::Success,
    })
}
