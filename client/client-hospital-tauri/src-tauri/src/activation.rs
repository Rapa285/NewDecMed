use anyhow::{anyhow, Context};
use tauri::{async_runtime::Mutex, State};
use umbral_pre::encrypt;

use crate::{
    current_fn,
    hospital_error::HospitalError,
    types::{
        AppState, CommandGlobalAdminAddActivationKeyResponse,
        CommandHospitalAdminAddActivationKeyResponse, HospitalPersonnelMetadata,
        HospitalPersonnelRole, MoveCallHospitalAdminAddActivationKeyPayload, ResponseStatus,
        SuccessResponse,
    },
    utils::{
        decode_hospital_personnel_id, decode_hospital_personnel_id_to_argon, encode_activation_key,
        encode_activation_key_from_keys_entry, generate_64_bytes_seed, generate_iota_keys_ed,
        get_global_admin_iota_address_from_keys_entry,
        get_global_admin_iota_key_pair_from_keys_entry, get_iota_address_from_keys_entry,
        get_iota_key_pair_from_keys_entry, get_pre_keys_from_keys_entry, parse_keys_entry,
        serde_serialize_to_base64,
    },
};
use base64::{engine::general_purpose::STANDARD, Engine as _};

#[tauri::command]
pub async fn global_admin_add_activation_key(
    state: State<'_, Mutex<AppState>>,
) -> Result<SuccessResponse<CommandGlobalAdminAddActivationKeyResponse>, HospitalError> {
    let state = state.lock().await;
    let keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    let (
        activation_key,
        activation_key_encoded,
        id,
        global_admin_iota_address,
        global_admin_iota_key_pair,
        hospital_admin_id_part_hash,
        hospital_admin_hospital_part_hash,
        hospital_name,
    ) = {
        let activation_key = uuid::Uuid::new_v4().to_string();
        let hospital_name = "Hospitalnya Jiwoo Yeppo";
        let hospital_part = "hos_ana";
        let id_part = "admin";
        let id = format!("{}@{}", id_part, hospital_part);

        let (hospital_admin_id_part_hash, hospital_admin_hospital_part_hash) =
            decode_hospital_personnel_id_to_argon(id.clone()).context(current_fn!())?;

        let activation_key_encoded =
            encode_activation_key(activation_key.clone(), id.clone()).context(current_fn!())?;

        let global_admin_iota_address =
            get_global_admin_iota_address_from_keys_entry(&keys_entry).context(current_fn!())?;
        let global_admin_iota_key_pair =
            get_global_admin_iota_key_pair_from_keys_entry(&keys_entry).context(current_fn!())?;

        (
            activation_key,
            activation_key_encoded,
            id,
            global_admin_iota_address,
            global_admin_iota_key_pair,
            hospital_admin_id_part_hash,
            hospital_admin_hospital_part_hash,
            hospital_name,
        )
    };

    let _ = state
        .move_call
        .global_admin_create_activation_key(
            activation_key_encoded,
            hospital_admin_id_part_hash,
            hospital_admin_hospital_part_hash,
            hospital_name.to_string(),
            global_admin_iota_address,
            global_admin_iota_key_pair,
        )
        .await
        .context(current_fn!())?;

    let res = CommandGlobalAdminAddActivationKeyResponse { activation_key, id };

    Ok(SuccessResponse {
        status: ResponseStatus::Success,
        data: res,
    })
}

#[tauri::command]
pub async fn hospital_admin_add_activation_key(
    state: State<'_, Mutex<AppState>>,
    personnel_id_part: String,
    role: String,
    pin: String,
) -> Result<SuccessResponse<CommandHospitalAdminAddActivationKeyResponse>, HospitalError> {
    let state = state.lock().await;
    let keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    let (
        admin_activation_key,
        hospital_admin_hospital_part,
        hospital_admin_pre_public_key,
        hospital_admin_iota_address,
        hospital_admin_iota_key_pair,
    ) = {
        let admin_activation_key =
            encode_activation_key_from_keys_entry(&keys_entry).context(current_fn!())?;

        let (_, hospital_admin_pre_public_key) =
            get_pre_keys_from_keys_entry(&keys_entry, pin.clone()).context(current_fn!())?;
        let (_, hospital_admin_hospital_part) =
            decode_hospital_personnel_id(keys_entry.id.clone().unwrap()).context(current_fn!())?;
        let hospital_admin_iota_address =
            get_iota_address_from_keys_entry(&keys_entry).context(current_fn!())?;
        let hospital_admin_iota_key_pair =
            get_iota_key_pair_from_keys_entry(&keys_entry, pin).context(current_fn!())?;

        (
            admin_activation_key,
            hospital_admin_hospital_part,
            hospital_admin_pre_public_key,
            hospital_admin_iota_address,
            hospital_admin_iota_key_pair,
        )
    };

    let (
        hospital_personnel_id,
        hospital_personnel_activation_key,
        metadata,
        hospital_personnel_id_part_hash,
    ) = {
        let hospital_personnel_id =
            format!("{}@{}", personnel_id_part, hospital_admin_hospital_part);
        let (hospital_personnel_id_part_hash, _) =
            decode_hospital_personnel_id_to_argon(hospital_personnel_id.clone())
                .context(current_fn!())?;

        let activation_key = uuid::Uuid::new_v4().to_string();

        let role_type = match role.as_str() {
            "AdministrativePersonnel" => HospitalPersonnelRole::AdministrativePersonnel,
            "MedicalPersonnel" => HospitalPersonnelRole::MedicalPersonnel,
            _ => return Err(HospitalError::Anyhow(anyhow!("Invalid role argument."))),
        };

        let hospital_personnel_metadata = HospitalPersonnelMetadata {
            activation_key: activation_key.clone(),
            id: hospital_personnel_id.clone(),
            role: role_type,
        };
        let hospital_personnel_metadata_bytes =
            serde_json::to_vec(&hospital_personnel_metadata).context(current_fn!())?;
        let (hospital_personnel_metadata_capsule, enc_hospital_personnel_metadata) = encrypt(
            &hospital_admin_pre_public_key,
            &hospital_personnel_metadata_bytes,
        )
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;

        let metadata = MoveCallHospitalAdminAddActivationKeyPayload {
            capsule: serde_serialize_to_base64(&hospital_personnel_metadata_capsule)
                .context(current_fn!())?,
            enc_metadata: STANDARD.encode(enc_hospital_personnel_metadata),
        };

        (
            hospital_personnel_id,
            activation_key,
            metadata,
            hospital_personnel_id_part_hash,
        )
    };

    let _ = state
        .move_call
        .hospital_admin_create_activation_key(
            admin_activation_key,
            serde_serialize_to_base64(&metadata).context(current_fn!())?,
            encode_activation_key(
                hospital_personnel_activation_key.clone(),
                hospital_personnel_id.clone(),
            )
            .context(current_fn!())?,
            hospital_personnel_id_part_hash,
            &role,
            hospital_admin_iota_address,
            hospital_admin_iota_key_pair,
        )
        .await
        .context(current_fn!())?;

    let data = CommandHospitalAdminAddActivationKeyResponse {
        activation_key: hospital_personnel_activation_key,
        id: hospital_personnel_id,
    };

    Ok(SuccessResponse {
        status: ResponseStatus::Success,
        data,
    })
}

#[tauri::command]
pub async fn activate_app(
    state: State<'_, Mutex<AppState>>,
    activation_key: String,
    id: String,
) -> Result<SuccessResponse<()>, HospitalError> {
    let state = state.lock().await;

    let (
        random_iota_address,
        random_iota_key_pair,
        hospital_personnel_id_part_hash,
        hospital_personnel_hospital_part_hash,
    ) = {
        let seed = generate_64_bytes_seed();
        let (random_iota_address, random_iota_key_pair) =
            generate_iota_keys_ed(&seed).context(current_fn!())?;
        let (hospital_personnel_id_part_hash, hospital_personnel_hospital_part_hash) =
            decode_hospital_personnel_id_to_argon(id.clone()).context(current_fn!())?;

        (
            random_iota_address,
            random_iota_key_pair,
            hospital_personnel_id_part_hash,
            hospital_personnel_hospital_part_hash,
        )
    };

    let _ = state
        .move_call
        .use_activation_key(
            encode_activation_key(activation_key.clone(), id.clone()).context(current_fn!())?,
            hospital_personnel_hospital_part_hash,
            hospital_personnel_id_part_hash,
            random_iota_address,
            random_iota_key_pair,
        )
        .await
        .context(current_fn!())?;

    let mut keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;
    keys_entry.activation_key = Some(activation_key);
    keys_entry.id = Some(id);

    state
        .keys_entry
        .set_secret(&serde_json::to_vec(&keys_entry).context(current_fn!())?)
        .context(current_fn!())?;

    Ok(SuccessResponse {
        status: ResponseStatus::Success,
        data: (),
    })
}
