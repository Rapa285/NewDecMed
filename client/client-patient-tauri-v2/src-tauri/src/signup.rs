use anyhow::{anyhow, Context};
use tauri::{async_runtime::Mutex, State};
use umbral_pre::encrypt;

use crate::{
    current_fn,
    patient_error::PatientError,
    types::{
        AdministrativeData, AppState, KeyNonce, PrivateAdministrativeData,
        PrivateAdministrativeMetadata, ResponseStatus, SuccessResponse,
    },
    utils::{
        aes_encrypt, aes_encrypt_custom_key, argon_hash, compute_pre_keys,
        compute_seed_from_seed_words, generate_iota_keys_ed, parse_keys_entry,
        serde_serialize_to_base64, sha_hash,
    },
};

use base64::{engine::general_purpose::STANDARD, Engine as _};

#[tauri::command]
pub async fn generate_mnemonic(
    state: State<'_, Mutex<AppState>>,
) -> Result<SuccessResponse<String>, PatientError> {
    let mut state = state.lock().await;

    let mnemonic = bip39::Mnemonic::generate(12).context(current_fn!())?;
    let words = mnemonic.words().collect::<Vec<&str>>().join(" ");

    state.signup_state.seed_words = Some(words.clone());

    Ok(SuccessResponse {
        status: ResponseStatus::Success,
        data: words,
    })
}

#[tauri::command]
pub async fn signup(
    state: State<'_, Mutex<AppState>>,
    id: String,
) -> Result<SuccessResponse<()>, PatientError> {
    let mut state = state.lock().await;

    let mut keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    let (seed, id_hash, patient_iota_address, patient_iota_key_pair, patient_pre_public_key) = {
        let seed = compute_seed_from_seed_words(
            &state
                .signup_state
                .seed_words
                .clone()
                .ok_or(anyhow!("Seed words not found").context(current_fn!()))?,
            &id,
        )
        .context(current_fn!())?;
        let (patient_iota_address, patient_iota_key_pair) =
            generate_iota_keys_ed(&seed).context(current_fn!())?;
        let (_, patient_pre_public_key) = compute_pre_keys(&seed[0..32]).context(current_fn!())?;
        let id_hash = argon_hash(id.clone()).context(current_fn!())?;

        (
            seed,
            id_hash,
            patient_iota_address,
            patient_iota_key_pair,
            patient_pre_public_key,
        )
    };

    let (private_administrative_data, private_administrative_metadata) = {
        // Construct private administrative data
        let private_administrative_data = PrivateAdministrativeData {
            id: id.clone(),
            name: None,
            ..Default::default()
        };
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

    let (
        enc_patient_iota_key_pair,
        enc_patient_pre_secret_key,
        patient_iota_key_pair_nonce,
        patient_pre_secret_key_nonce,
    ) = {
        // Encrypt PRE secret key
        let (enc_patient_pre_secret_key, patient_pre_secret_key_nonce) = aes_encrypt_custom_key(
            sha_hash(
                state
                    .signup_state
                    .pin
                    .as_ref()
                    .ok_or(anyhow!("PIN not found on signup state").context(current_fn!()))?
                    .as_bytes(),
            )
            .as_slice(),
            &seed[0..32],
        )
        .context(current_fn!())?;

        // Encrypt IOTA keypair
        let (enc_patient_iota_key_pair, patient_iota_key_pair_nonce) = aes_encrypt_custom_key(
            sha_hash(
                state
                    .signup_state
                    .pin
                    .as_ref()
                    .ok_or(anyhow!("PIN not found on signup state").context(current_fn!()))?
                    .as_bytes(),
            )
            .as_slice(),
            patient_iota_key_pair
                .encode()
                .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?
                .as_bytes(),
        )
        .context(current_fn!())?;

        (
            enc_patient_iota_key_pair,
            enc_patient_pre_secret_key,
            patient_iota_key_pair_nonce,
            patient_pre_secret_key_nonce,
        )
    };

    // Process tx
    let _ = state
        .move_call
        .signup(
            id_hash,
            serde_serialize_to_base64(&private_administrative_metadata).context(current_fn!())?,
            patient_iota_address,
            patient_iota_key_pair,
        )
        .await
        .context(current_fn!())?;

    keys_entry.id = Some(id);
    keys_entry.iota_address = Some(patient_iota_address.to_string());
    keys_entry.iota_key_pair = Some(STANDARD.encode(enc_patient_iota_key_pair));
    keys_entry.iota_nonce = Some(STANDARD.encode(patient_iota_key_pair_nonce));
    keys_entry.pre_nonce = Some(STANDARD.encode(patient_pre_secret_key_nonce));
    keys_entry.pre_public_key =
        Some(serde_serialize_to_base64(&patient_pre_public_key).context(current_fn!())?);
    keys_entry.pre_secret_key = Some(STANDARD.encode(enc_patient_pre_secret_key));

    let keys_entry = serde_json::to_vec(&keys_entry).context(current_fn!())?;
    state
        .keys_entry
        .set_secret(&keys_entry)
        .context(current_fn!())?;

    state.administrative_data = Some(AdministrativeData {
        private: private_administrative_data,
    });

    // drop SignupState from state
    state.signup_state.pin = None;
    state.signup_state.seed_words = None;

    Ok(SuccessResponse {
        data: (),
        status: ResponseStatus::Success,
    })
}
