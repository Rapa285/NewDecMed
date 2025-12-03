use anyhow::{anyhow, Context};
use tauri::{async_runtime::Mutex, State};

use crate::{
    current_fn,
    patient_error::PatientError,
    types::{AppState, ResponseStatus, SuccessResponse},
    utils::{
        aes_encrypt_custom_key, compute_pre_keys, compute_seed_from_seed_words,
        generate_iota_keys_ed, parse_keys_entry, serde_serialize_to_base64, sha_hash,
        validate_by_regex,
    },
};
use base64::{engine::general_purpose::STANDARD, Engine as _};

#[tauri::command]
pub async fn signin(
    state: State<'_, Mutex<AppState>>,
    seed_words: String,
    id: String,
) -> Result<SuccessResponse<()>, PatientError> {
    let mut state = state.lock().await;
    let mut keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    if !validate_by_regex(&id, "^[0-9]{16}$").context(current_fn!())? {
        return Err(PatientError::Anyhow(anyhow!("Invalid id")));
    }

    let (
        pin,
        patient_iota_address,
        patient_pre_public_key,
        enc_patient_iota_key_pair,
        enc_patient_pre_secret_key,
        patient_pre_secret_key_nonce,
        patient_iota_key_pair_nonce,
    ) = {
        let pin = state
            .signin_state
            .pin
            .clone()
            .ok_or(anyhow!("PIN not found on signin state").context(current_fn!()))?;
        let seed = compute_seed_from_seed_words(&seed_words, &id).context(current_fn!())?;
        let (patient_iota_address, patient_iota_key_pair) =
            generate_iota_keys_ed(&seed).context(current_fn!())?;
        let (_, patient_pre_public_key) = compute_pre_keys(&seed[0..32]).context(current_fn!())?;

        let (enc_patient_pre_secret_key, patient_pre_secret_key_nonce) =
            aes_encrypt_custom_key(&sha_hash(pin.as_bytes()), &seed[0..32])
                .context(current_fn!())?;
        let (enc_patient_iota_key_pair, patient_iota_key_pair_nonce) = aes_encrypt_custom_key(
            &sha_hash(pin.as_bytes()),
            patient_iota_key_pair
                .encode()
                .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?
                .as_bytes(),
        )
        .context(current_fn!())?;

        (
            pin,
            patient_iota_address,
            patient_pre_public_key,
            enc_patient_iota_key_pair,
            enc_patient_pre_secret_key,
            patient_pre_secret_key_nonce,
            patient_iota_key_pair_nonce,
        )
    };

    let () = {};

    let is_registered = state
        .move_call
        .is_account_registered(patient_iota_address)
        .await
        .context(current_fn!())?;

    if !is_registered {
        return Err(PatientError::Anyhow(anyhow!("Account not found")));
    }

    keys_entry.id = Some(id);
    keys_entry.iota_address = Some(patient_iota_address.to_string());
    keys_entry.iota_key_pair = Some(STANDARD.encode(enc_patient_iota_key_pair));
    keys_entry.pre_secret_key = Some(STANDARD.encode(enc_patient_pre_secret_key));
    keys_entry.pre_public_key =
        Some(serde_serialize_to_base64(&patient_pre_public_key).context(current_fn!())?);
    keys_entry.pre_nonce = Some(STANDARD.encode(patient_pre_secret_key_nonce));
    keys_entry.iota_nonce = Some(STANDARD.encode(patient_iota_key_pair_nonce));
    let keys_entry = serde_json::to_vec(&keys_entry).context(current_fn!())?;
    state
        .keys_entry
        .set_secret(&keys_entry)
        .context(current_fn!())?;

    state.auth_state.session_pin = Some(pin);

    // drop SigninState form state
    state.signin_state.pin = None;

    Ok(SuccessResponse {
        status: ResponseStatus::Success,
        data: (),
    })
}
