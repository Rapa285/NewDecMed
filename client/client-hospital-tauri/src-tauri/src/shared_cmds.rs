use anyhow::{anyhow, Context};
use tauri::{async_runtime::Mutex, State};
use umbral_pre::{decrypt_original, encrypt, Capsule};

use crate::{
    current_fn,
    hospital_error::HospitalError,
    types::{
        AdministrativeData, AppState, CommandGetProfileResponseData, CommandUpdateProfileArgs,
        HospitalPersonnelRole, KeyNonce, PrivateAdministrativeData, PrivateAdministrativeMetadata,
        PublicAdministrativeData, ResponseStatus, SuccessResponse,
    },
    utils::{
        aes_decrypt, aes_encrypt, decode_hospital_personnel_id_to_argon,
        encode_activation_key_from_keys_entry, generate_64_bytes_seed, generate_iota_keys_ed,
        get_iota_address_from_keys_entry, get_iota_key_pair_from_keys_entry,
        get_pre_keys_from_keys_entry, parse_keys_entry, serde_deserialize_from_base64,
        serde_serialize_to_base64, validate_by_regex,
    },
};

use base64::{engine::general_purpose::STANDARD, Engine as _};

#[tauri::command]
pub async fn validate_pin(
    state: State<'_, Mutex<AppState>>,
    pin: String,
    auth_type: String,
) -> Result<SuccessResponse<()>, HospitalError> {
    let mut state = state.lock().await;

    if !validate_by_regex(&pin, "^[0-9]{6}$").context(current_fn!())? {
        return Err(HospitalError::Anyhow(anyhow!("Invalid PIN")));
    }

    match auth_type.as_str() {
        "Signin" => {
            state.signin_state.pin = Some(pin);
        }
        "Signup" => {
            state.signup_state.pin = Some(pin);
        }
        _ => {
            return Err(HospitalError::Anyhow(
                anyhow!("Invalid arg auth_type").context(current_fn!()),
            ))
        }
    };

    Ok(SuccessResponse {
        status: ResponseStatus::Success,
        data: (),
    })
}

#[tauri::command]
pub async fn validate_confirm_pin(
    state: State<'_, Mutex<AppState>>,
    confirm_pin: String,
    auth_type: String,
) -> Result<SuccessResponse<()>, HospitalError> {
    let state = state.lock().await;

    if !validate_by_regex(&confirm_pin, "^[0-9]{6}$").context(current_fn!())? {
        return Err(HospitalError::Anyhow(anyhow!("Invalid confirm PIN")));
    }

    match auth_type.as_str() {
        "Signin" => {
            if *state.signin_state.pin.as_ref().unwrap() != confirm_pin {
                return Err(HospitalError::Anyhow(anyhow!(
                    "PIN and confirm PIN must be same"
                )));
            }
        }
        "Signup" => {
            if *state.signup_state.pin.as_ref().unwrap() != confirm_pin {
                return Err(HospitalError::Anyhow(anyhow!(
                    "PIN and confirm PIN must be same"
                )));
            }
        }
        _ => {
            return Err(HospitalError::Anyhow(
                anyhow!("Invalid arg auth_type").context(current_fn!()),
            ))
        }
    };

    Ok(SuccessResponse {
        status: ResponseStatus::Success,
        data: (),
    })
}

#[tauri::command]
pub async fn get_profile(
    state: State<'_, Mutex<AppState>>,
) -> Result<SuccessResponse<CommandGetProfileResponseData>, HospitalError> {
    let mut state = state.lock().await;
    let keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    let (
        activation_key,
        hospital_personnel_iota_address,
        hospital_personnel_iota_key_pair,
        hospital_personnel_pre_public_key,
        hospital_personnel_pre_secret_key,
    ) = {
        let activation_key =
            encode_activation_key_from_keys_entry(&keys_entry).context(current_fn!())?;

        let pin = state
            .auth_state
            .session_pin
            .clone()
            .ok_or(anyhow!("Session PIN not found").context(current_fn!()))?;
        let hospital_personnel_iota_address =
            get_iota_address_from_keys_entry(&keys_entry).context(current_fn!())?;
        let hospital_personnel_iota_key_pair =
            get_iota_key_pair_from_keys_entry(&keys_entry, pin.clone()).context(current_fn!())?;
        let (hospital_personnel_pre_secret_key, hospital_personnel_pre_public_key) =
            get_pre_keys_from_keys_entry(&keys_entry, pin.clone()).context(current_fn!())?;

        (
            activation_key,
            hospital_personnel_iota_address,
            hospital_personnel_iota_key_pair,
            hospital_personnel_pre_public_key,
            hospital_personnel_pre_secret_key,
        )
    };

    let (hospital_personnel_administrative_metadata, role, hospital_metadata) = state
        .move_call
        .get_account_info(activation_key, hospital_personnel_iota_address)
        .await
        .context(current_fn!())?;

    let administrative_metadata = hospital_personnel_administrative_metadata
        .ok_or(anyhow!("Administrative metadata not found").context(current_fn!()))?;

    let private_administrative_metadata: PrivateAdministrativeMetadata =
        serde_deserialize_from_base64(administrative_metadata.private_metadata)
            .context(current_fn!())?;
    let public_administrative_data: PublicAdministrativeData =
        serde_deserialize_from_base64(administrative_metadata.public_metadata)
            .context(current_fn!())?;

    let private_administrative_data_capsule: Capsule =
        serde_deserialize_from_base64(private_administrative_metadata.capsule)
            .context(current_fn!())?;
    let private_administrative_data_key_nonce = decrypt_original(
        &hospital_personnel_pre_secret_key,
        &private_administrative_data_capsule,
        &STANDARD
            .decode(private_administrative_metadata.enc_key_nonce)
            .context(current_fn!())?,
    )
    .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;
    let private_administrative_data_key_nonce: KeyNonce =
        serde_json::from_slice(&private_administrative_data_key_nonce).context(current_fn!())?;
    let private_administrative_data = aes_decrypt(
        &STANDARD
            .decode(private_administrative_metadata.enc_data)
            .context(current_fn!())?,
        &STANDARD
            .decode(private_administrative_data_key_nonce.key)
            .context(current_fn!())?,
        &STANDARD
            .decode(private_administrative_data_key_nonce.nonce)
            .context(current_fn!())?,
    )?;
    let private_administrative_data: PrivateAdministrativeData =
        serde_json::from_slice(&private_administrative_data).context(current_fn!())?;

    let hospital_personnel_id_hash = {
        let (hospital_personnel_id_part_hash, hospital_personnel_hospital_part_hash) =
            decode_hospital_personnel_id_to_argon(private_administrative_data.id.clone())
                .context(current_fn!())?;
        let hospital_personnel_id_hash = format!(
            "{}@{}",
            hospital_personnel_id_part_hash, hospital_personnel_hospital_part_hash
        );
        hospital_personnel_id_hash
    };

    let data = CommandGetProfileResponseData {
        hospital: hospital_metadata.name.clone(),
        id: private_administrative_data.id.clone(),
        id_hash: hospital_personnel_id_hash,
        iota_address: hospital_personnel_iota_address.to_string(),
        iota_key_pair: hospital_personnel_iota_key_pair
            .encode()
            .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?,
        name: public_administrative_data.name.clone(),
        pre_public_key: serde_serialize_to_base64(&hospital_personnel_pre_public_key)
            .context(current_fn!())?,
        role,
    };

    state.administrative_data = Some(AdministrativeData {
        private: private_administrative_data,
        public: public_administrative_data,
    });

    Ok(SuccessResponse {
        status: ResponseStatus::Success,
        data,
    })
}

#[tauri::command]
pub async fn update_profile(
    state: State<'_, Mutex<AppState>>,
    data: CommandUpdateProfileArgs,
) -> Result<SuccessResponse<()>, HospitalError> {
    let mut state = state.lock().await;
    let keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    // Validate data
    if !validate_by_regex(&data.name, "^[a-zA-Z ]{2,100}$").context(current_fn!())? {
        return Err(HospitalError::Anyhow(anyhow!(
            "Invalid args: data.name is invalid"
        )));
    }

    let (
        activation_key,
        hospital_personnel_iota_key_pair,
        hospital_personnel_iota_address,
        hospital_personnel_pre_public_key,
    ) = {
        let activation_key =
            encode_activation_key_from_keys_entry(&keys_entry).context(current_fn!())?;

        let pin = state
            .auth_state
            .session_pin
            .clone()
            .ok_or(anyhow!("Session PIN not found").context(current_fn!()))?;
        let hospital_personnel_iota_key_pair =
            get_iota_key_pair_from_keys_entry(&keys_entry, pin.clone()).context(current_fn!())?;
        let hospital_personnel_iota_address =
            get_iota_address_from_keys_entry(&keys_entry).context(current_fn!())?;
        let (_, hospital_personnel_pre_public_key) =
            get_pre_keys_from_keys_entry(&keys_entry, pin).context(current_fn!())?;

        (
            activation_key,
            hospital_personnel_iota_key_pair,
            hospital_personnel_iota_address,
            hospital_personnel_pre_public_key,
        )
    };

    let (private_administrative_data, private_administrative_metadata) = {
        // Construct private administrative data
        let private_administrative_data = state
            .administrative_data
            .as_ref()
            .ok_or(anyhow!("Administrative data not found on state").context(current_fn!()))?
            .private
            .clone();
        let (
            enc_private_administrative_data,
            private_administrative_data_key,
            private_administrative_data_nonce,
        ) = aes_encrypt(&serde_json::to_vec(&private_administrative_data).context(current_fn!())?)
            .context(current_fn!())?;

        let private_administrative_data_key_nonce = KeyNonce {
            key: STANDARD.encode(private_administrative_data_key),
            nonce: STANDARD.encode(private_administrative_data_nonce),
        };
        let (
            private_administrative_data_key_nonce_capsule,
            enc_private_administrative_data_key_nonce,
        ) = encrypt(
            &hospital_personnel_pre_public_key,
            &serde_json::to_vec(&private_administrative_data_key_nonce).context(current_fn!())?,
        )
        .unwrap();

        let private_administrative_metadata = PrivateAdministrativeMetadata {
            capsule: serde_serialize_to_base64(&private_administrative_data_key_nonce_capsule)
                .context(current_fn!())?,
            enc_data: STANDARD.encode(enc_private_administrative_data),
            enc_key_nonce: STANDARD.encode(enc_private_administrative_data_key_nonce),
        };

        (private_administrative_data, private_administrative_metadata)
    };

    let public_administrative_data = {
        // Construct public administrative data
        let mut public_administrative_data = state
            .administrative_data
            .as_ref()
            .ok_or(anyhow!("Administrative data not found on state").context(current_fn!()))?
            .public
            .clone();
        public_administrative_data.name = Some(data.name);

        public_administrative_data
    };

    let _ = state
        .move_call
        .update_administrative_metadata(
            activation_key,
            serde_serialize_to_base64(&private_administrative_metadata).context(current_fn!())?,
            serde_serialize_to_base64(&public_administrative_data).context(current_fn!())?,
            hospital_personnel_iota_address,
            hospital_personnel_iota_key_pair,
        )
        .await
        .context(current_fn!())?;

    state.administrative_data = Some(AdministrativeData {
        private: private_administrative_data,
        public: public_administrative_data,
    });

    Ok(SuccessResponse {
        data: (),
        status: ResponseStatus::Success,
    })
}

/**
 * Error context:
 * $<0>$: redirect to activation page
 * $<1>$: redirect to signup page
 * $<2>$: redirect to signin page
 * $<3>$: redirect to complete-profile page
 * $<4>$: redirect to pin page
 */
#[tauri::command]
pub async fn auth_status(
    state: State<'_, Mutex<AppState>>,
) -> Result<SuccessResponse<Option<HospitalPersonnelRole>>, HospitalError> {
    let state = state.lock().await;
    let keys_entry = parse_keys_entry(
        &state
            .keys_entry
            .get_secret()
            .context("$<0>$")
            .context(current_fn!())?,
    )
    .context("$<0>$")
    .context(current_fn!())?;

    if keys_entry.activation_key.is_none() || keys_entry.id.is_none() {
        return Err(HospitalError::Anyhow(
            anyhow!("Activation key or id not found").context("$<0>$"),
        ));
    }

    // With the following iota call we can check if the activation key exist
    // and id is registered

    let (
        activation_key,
        random_iota_address,
        hospital_personnel_id_part_hash,
        hospital_personnel_hospital_part_hash,
    ) = {
        let id = keys_entry
            .id
            .clone()
            .ok_or(anyhow!("Id not found on keys entry").context(current_fn!()))?;
        let activation_key =
            encode_activation_key_from_keys_entry(&keys_entry).context(current_fn!())?;
        let seed = generate_64_bytes_seed();
        let (random_iota_address, _) = generate_iota_keys_ed(&seed)
            .context("$<0>$")
            .context(current_fn!())?;
        let (hospital_personnel_id_part_hash, hospital_personnel_hospital_part_hash) =
            decode_hospital_personnel_id_to_argon(id)
                .context("$<0>$")
                .context(current_fn!())?;

        (
            activation_key,
            random_iota_address,
            hospital_personnel_id_part_hash,
            hospital_personnel_hospital_part_hash,
        )
    };

    let (account_state, role) = state
        .move_call
        .get_account_state(
            activation_key,
            hospital_personnel_hospital_part_hash,
            hospital_personnel_id_part_hash,
            random_iota_address,
        )
        .await
        .context("$<0>$")
        .context(current_fn!())?;

    match account_state {
        0 => {
            return Err(HospitalError::Anyhow(
                anyhow!("Activation needed").context("$<0>$"),
            ))
        }
        1 => {
            return Err(HospitalError::Anyhow(
                anyhow!("Signup needed").context("$<1>$"),
            ))
        }
        _ => {}
    };

    if keys_entry.iota_key_pair.is_none() || keys_entry.pre_secret_key.is_none() {
        return Err(HospitalError::Anyhow(
            anyhow!("IOTA Key Pair and PRE Secret Key not found").context("$<2>$"),
        ));
    }

    if account_state == 2 {
        return Err(HospitalError::Anyhow(
            anyhow!("Profile completion needed").context("$<3>$"),
        ));
    }

    if state.auth_state.session_pin.is_none() {
        return Err(HospitalError::Anyhow(
            anyhow!("Session PIN not found").context("$<4>$"),
        ));
    }

    Ok(SuccessResponse {
        data: role,
        status: ResponseStatus::Success,
    })
}
