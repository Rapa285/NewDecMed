use std::str::FromStr;

use anyhow::{anyhow, Context};
use bip39::Mnemonic;
use serde_json::{json, Value};
use tauri::{async_runtime::Mutex, State};
use umbral_pre::{decrypt_original, encrypt};

use crate::{
    current_fn,
    patient_error::PatientError,
    types::{
        AdministrativeData, AppState, CommandUpdateProfileInput, KeyNonce,
        PrivateAdministrativeData, PrivateAdministrativeMetadata, ResponseStatus, SuccessResponse,
    },
    utils::{
        aes_decrypt, aes_encrypt, argon_hash, generate_64_bytes_seed, generate_iota_keys_ed,
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
) -> Result<SuccessResponse<()>, PatientError> {
    let mut state = state.lock().await;

    if !validate_by_regex(&pin, "^[0-9]{6}$").context(current_fn!())? {
        return Err(PatientError::Anyhow(anyhow!("Invalid PIN")));
    }

    match auth_type.as_str() {
        "Signin" => {
            state.signin_state.pin = Some(pin);
        }
        "Signup" => {
            state.signup_state.pin = Some(pin);
        }
        _ => return Err(PatientError::Anyhow(anyhow!("Invalid auth_type arg"))),
    };

    Ok(SuccessResponse {
        data: (),
        status: ResponseStatus::Success,
    })
}

#[tauri::command]
pub async fn validate_confirm_pin(
    state: State<'_, Mutex<AppState>>,
    confirm_pin: String,
    auth_type: String,
) -> Result<SuccessResponse<()>, PatientError> {
    let state = state.lock().await;

    if !validate_by_regex(&confirm_pin, "^[0-9]{6}$").context(current_fn!())? {
        return Err(PatientError::Anyhow(anyhow!("Invalid confirm pin")));
    }

    match auth_type.as_str() {
        "Signin" => {
            if *state
                .signin_state
                .pin
                .as_ref()
                .ok_or(anyhow!("PIN not found on signin_state").context(current_fn!()))?
                != confirm_pin
            {
                return Err(PatientError::Anyhow(anyhow!(
                    "Confirm PIN and PIN must be same"
                )));
            }
        }
        "Signup" => {
            if *state
                .signup_state
                .pin
                .as_ref()
                .ok_or(anyhow!("PIN not found on signup_state").context(current_fn!()))?
                != confirm_pin
            {
                return Err(PatientError::Anyhow(anyhow!(
                    "Confirm PIN and PIN must be same"
                )));
            }
        }
        _ => {
            return Err(PatientError::Anyhow(
                anyhow!("Invalid auth_type arg").context(current_fn!()),
            ))
        }
    };

    Ok(SuccessResponse {
        data: (),
        status: ResponseStatus::Success,
    })
}

#[tauri::command]
pub async fn validate_seed_words(
    state: State<'_, Mutex<AppState>>,
    seed_words: String,
    auth_type: String,
) -> Result<SuccessResponse<()>, PatientError> {
    let state = state.lock().await;

    let seed_words = Mnemonic::from_str(&seed_words)
        .context(current_fn!())?
        .words()
        .collect::<Vec<&str>>()
        .join(" ");

    match auth_type.as_str() {
        "Signin" => {}
        "Signup" => {
            if *state.signup_state.seed_words.as_ref().unwrap() != seed_words {
                return Err(PatientError::Anyhow(anyhow!("Invalid seed words")));
            }
        }
        _ => {
            return Err(PatientError::Anyhow(
                anyhow!("Invalid auth_type arg").context(current_fn!()),
            ))
        }
    };

    Ok(SuccessResponse {
        data: (),
        status: ResponseStatus::Success,
    })
}

#[tauri::command]
pub async fn is_session_pin_exist(
    state: State<'_, Mutex<AppState>>,
) -> Result<SuccessResponse<()>, PatientError> {
    let state = state.lock().await;

    match state.auth_state.session_pin {
        Some(_) => {
            return Ok(SuccessResponse {
                data: (),
                status: ResponseStatus::Success,
            })
        }
        None => return Err(PatientError::Anyhow(anyhow!("Session PIN not found"))),
    }
}

#[tauri::command]
pub async fn get_profile(
    state: State<'_, Mutex<AppState>>,
) -> Result<SuccessResponse<Value>, PatientError> {
    let mut state = state.lock().await;
    let keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    let (patient_iota_address, patient_pre_secret_key, patient_pre_public_key) = {
        let pin = state
            .auth_state
            .session_pin
            .clone()
            .ok_or(anyhow!("Session PIN not found").context(current_fn!()))?;
        let patient_iota_address =
            get_iota_address_from_keys_entry(&keys_entry).context(current_fn!())?;
        let (patient_pre_secret_key, patient_pre_public_key) =
            get_pre_keys_from_keys_entry(&keys_entry, pin).context(current_fn!())?;

        (
            patient_iota_address,
            patient_pre_secret_key,
            patient_pre_public_key,
        )
    };

    let private_administrative_data = state
        .move_call
        .get_account_info(patient_iota_address)
        .await
        .context(current_fn!())?;

    let private_administrative_metadata: PrivateAdministrativeMetadata =
        serde_deserialize_from_base64(private_administrative_data.private_metadata)
            .context(current_fn!())?;

    let key_nonce = decrypt_original(
        &patient_pre_secret_key,
        &serde_deserialize_from_base64(private_administrative_metadata.capsule)
            .context(current_fn!())?,
        &STANDARD
            .decode(private_administrative_metadata.enc_key_nonce)
            .context(current_fn!())?,
    )
    .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;
    let key_nonce: KeyNonce = serde_json::from_slice(&key_nonce).context(current_fn!())?;

    let private_administrative_data = aes_decrypt(
        &STANDARD
            .decode(private_administrative_metadata.enc_data)
            .context(current_fn!())?,
        &STANDARD.decode(key_nonce.key).context(current_fn!())?,
        &STANDARD.decode(key_nonce.nonce).context(current_fn!())?,
    )
    .context(current_fn!())?;
    let private_administrative_data: PrivateAdministrativeData =
        serde_json::from_slice(&private_administrative_data).context(current_fn!())?;

    let data = json!({
        "id": keys_entry.id,
        "idHash": argon_hash(private_administrative_data.id.clone()).context(current_fn!())?,
        "iotaAddress": patient_iota_address.to_string(),
        "prePublicKey": serde_serialize_to_base64(&patient_pre_public_key)
            .context(current_fn!())?,
        "name": private_administrative_data.name.clone(),
        "birthPlace": private_administrative_data.birth_place,
        "dateOfBirth": private_administrative_data.date_of_birth,
        "gender": private_administrative_data.gender,
        "religion": private_administrative_data.religion,
        "education": private_administrative_data.education,
        "occupation": private_administrative_data.occupation,
        "maritalStatus": private_administrative_data.marital_status,
    });

    state.administrative_data = Some(AdministrativeData {
        private: private_administrative_data,
    });

    Ok(SuccessResponse {
        status: ResponseStatus::Success,
        data,
    })
}

#[tauri::command]
pub async fn update_profile(
    state: State<'_, Mutex<AppState>>,
    data: CommandUpdateProfileInput,
) -> Result<SuccessResponse<()>, PatientError> {
    let mut state = state.lock().await;
    let keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    // Validate data
    if !validate_by_regex(&data.name, "^[a-zA-Z ]{2,100}$").context(current_fn!())? {
        return Err(PatientError::Anyhow(anyhow!(
            "Invalid args: data.name is invalid"
        )));
    }

    let (patient_iota_key_pair, patient_iota_address, patient_pre_public_key) = {
        let pin = state
            .auth_state
            .session_pin
            .clone()
            .ok_or(anyhow!("Session PIN not found").context(current_fn!()))?;
        let patient_iota_key_pair =
            get_iota_key_pair_from_keys_entry(&keys_entry, pin.clone()).context(current_fn!())?;
        let patient_iota_address =
            get_iota_address_from_keys_entry(&keys_entry).context(current_fn!())?;
        let (_, patient_pre_public_key) =
            get_pre_keys_from_keys_entry(&keys_entry, pin).context(current_fn!())?;

        (
            patient_iota_key_pair,
            patient_iota_address,
            patient_pre_public_key,
        )
    };

    let (private_administrative_data, private_administrative_metadata) = {
        // Construct private administrative data
        let mut private_administrative_data = state
            .administrative_data
            .clone()
            .unwrap_or(AdministrativeData {
                private: PrivateAdministrativeData {
                    ..Default::default()
                },
            })
            .private;
        private_administrative_data.name = Some(data.name);
        private_administrative_data.birth_place = Some(data.birth_place);
        private_administrative_data.date_of_birth = Some(data.date_of_birth);
        private_administrative_data.education = Some(data.education);
        private_administrative_data.gender = Some(data.gender);
        private_administrative_data.marital_status = Some(data.marital_status);
        private_administrative_data.occupation = Some(data.occupation);
        private_administrative_data.religion = Some(data.religion);
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
            &patient_pre_public_key,
            &serde_json::to_vec(&private_administrative_data_key_nonce).context(current_fn!())?,
        )
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;

        let private_administrative_metadata = PrivateAdministrativeMetadata {
            capsule: serde_serialize_to_base64(&private_administrative_data_key_nonce_capsule)
                .context(current_fn!())?,
            enc_data: STANDARD.encode(enc_private_administrative_data),
            enc_key_nonce: STANDARD.encode(enc_private_administrative_data_key_nonce),
        };

        (private_administrative_data, private_administrative_metadata)
    };

    let _ = state
        .move_call
        .update_administrative_metadata(
            serde_serialize_to_base64(&private_administrative_metadata).context(current_fn!())?,
            patient_iota_address,
            patient_iota_key_pair,
        )
        .await
        .context(current_fn!())?;

    state.administrative_data = Some(AdministrativeData {
        private: private_administrative_data,
    });

    Ok(SuccessResponse {
        data: (),
        status: ResponseStatus::Success,
    })
}

/**
 * Error context:
 * $<0>$: redirect to auth page (signin/signup)
 * $<1>$: redirect to complete-profile page
 * $<2>$: redirect to pin page
 */
#[tauri::command]
pub async fn auth_status(state: State<'_, Mutex<AppState>>) -> Result<(), PatientError> {
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

    if keys_entry.id.is_none() {
        return Err(PatientError::Anyhow(
            anyhow!("Id not found on keys entry").context("$<0>$"),
        ));
    }

    // With the following iota call we can check if the activation key exist
    // and id is registered

    let (patient_id, random_iota_address) = {
        let id = keys_entry.id.clone().unwrap();
        let seed = generate_64_bytes_seed();
        let (random_iota_address, _) = generate_iota_keys_ed(&seed)
            .context("$<0>$")
            .context(current_fn!())?;
        let patient_id = argon_hash(id).context(current_fn!())?;

        (patient_id, random_iota_address)
    };

    let account_state = state
        .move_call
        .get_account_state(patient_id, random_iota_address)
        .await
        .context("$<0>$")
        .context(current_fn!())?;

    match account_state {
        0 => {
            return Err(PatientError::Anyhow(
                anyhow!("Signup needed").context("$<0>$"),
            ))
        }
        _ => {}
    };

    if keys_entry.iota_key_pair.is_none() || keys_entry.pre_secret_key.is_none() {
        return Err(PatientError::Anyhow(
            anyhow!("IOTA Key Pair and PRE Secret Key not found").context("$<0>$"),
        ));
    }

    if account_state == 1 {
        return Err(PatientError::Anyhow(
            anyhow!("Need profile completion").context("$<1>$"),
        ));
    }

    if state.auth_state.session_pin.is_none() {
        return Err(PatientError::Anyhow(
            anyhow!("Session PIN not found").context("$<2>$"),
        ));
    }

    Ok(())
}
