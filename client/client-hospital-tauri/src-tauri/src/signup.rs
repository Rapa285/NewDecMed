use anyhow::{anyhow, Context};
use bip39::Mnemonic;
use tauri::{async_runtime::Mutex, State};
use umbral_pre::encrypt;

use crate::{
    current_fn,
    hospital_error::HospitalError,
    types::{
        AdministrativeData, AppState, KeyNonce, PrivateAdministrativeData,
        PrivateAdministrativeMetadata, PublicAdministrativeData, ResponseStatus, SuccessResponse,
    },
    utils::{
        aes_encrypt, aes_encrypt_custom_key, compute_pre_keys, compute_seed_from_seed_words,
        decode_hospital_personnel_id_to_argon, encode_activation_key_from_keys_entry,
        generate_iota_keys_ed, parse_keys_entry, serde_serialize_to_base64, sha_hash,
    },
};
use base64::{engine::general_purpose::STANDARD, Engine as _};

#[tauri::command]
pub async fn generate_mnemonic(
    state: State<'_, Mutex<AppState>>,
) -> Result<SuccessResponse<String>, HospitalError> {
    let mut state = state.lock().await;

    let mnemonic = Mnemonic::generate(12).context(current_fn!())?;
    let words = mnemonic.words().collect::<Vec<&str>>().join(" ");

    state.signup_state.seed_words = Some(words.clone());

    Ok(SuccessResponse {
        data: words,
        status: ResponseStatus::Success,
    })
}

#[tauri::command]
pub async fn is_signed_up(
    state: State<'_, Mutex<AppState>>,
) -> Result<SuccessResponse<()>, HospitalError> {
    let state = state.lock().await;

    match state.auth_state.is_signed_up {
        true => {
            return Ok(SuccessResponse {
                status: ResponseStatus::Success,
                data: (),
            })
        }
        false => return Err(HospitalError::Anyhow(anyhow!("Not registered"))),
    }
}

#[tauri::command]
pub async fn signup(
    state: State<'_, Mutex<AppState>>,
    seed_words: String,
) -> Result<SuccessResponse<()>, HospitalError> {
    let mut state = state.lock().await;
    let mut keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    if *state
        .signup_state
        .seed_words
        .as_ref()
        .ok_or(anyhow!("Seed words not found on signup state").context(current_fn!()))?
        != seed_words
    {
        return Err(HospitalError::Anyhow(anyhow!("Invalid seed words")));
    }

    let (
        id,
        pin,
        seed,
        activation_key,
        hospital_personnnel_id_part_hash,
        hospital_personnnel_hospital_part_hash,
        hospital_personnel_iota_address,
        hospital_personnel_iota_key_pair,
        hospital_personnel_pre_public_key,
    ) = {
        let id = keys_entry
            .id
            .clone()
            .ok_or(anyhow!("Id not found on keys entry").context(current_fn!()))?;
        let pin = state
            .signup_state
            .pin
            .clone()
            .ok_or(anyhow!("PIN not found on signup state").context(current_fn!()))?;
        let activation_key =
            encode_activation_key_from_keys_entry(&keys_entry).context(current_fn!())?;
        let (hospital_personnnel_id_part_hash, hospital_personnnel_hospital_part_hash) =
            decode_hospital_personnel_id_to_argon(id.clone())?;
        let seed =
            compute_seed_from_seed_words(&state.signup_state.seed_words.as_ref().unwrap(), &id)
                .context(current_fn!())?;
        let (hospital_personnel_iota_address, hospital_personnel_iota_key_pair) =
            generate_iota_keys_ed(&seed).context(current_fn!())?;
        let (_, hospital_personnel_pre_public_key) =
            compute_pre_keys(&seed[0..32]).context(current_fn!())?;

        (
            id,
            pin,
            seed,
            activation_key,
            hospital_personnnel_id_part_hash,
            hospital_personnnel_hospital_part_hash,
            hospital_personnel_iota_address,
            hospital_personnel_iota_key_pair,
            hospital_personnel_pre_public_key,
        )
    };

    let (
        private_administrative_data,
        private_administrative_metadata,
        public_administrative_data,
        enc_hospital_personnel_pre_secret_key,
        hospital_personnel_pre_secret_key_nonce,
        enc_hospital_personnel_iota_key_pair,
        hospital_personnel_iota_keypair_nonce,
    ) = {
        // Construct private administrative data
        let private_administrative_data = PrivateAdministrativeData { id: id.clone() };
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
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;

        let private_administrative_metadata = PrivateAdministrativeMetadata {
            capsule: serde_serialize_to_base64(&private_administrative_data_key_nonce_capsule)
                .context(current_fn!())?,
            enc_data: STANDARD.encode(enc_private_administrative_data),
            enc_key_nonce: STANDARD.encode(enc_private_administrative_data_key_nonce),
        };

        // Construct public administrative data
        let public_administrative_data = PublicAdministrativeData { name: None };

        // Encrypt PRE secret key
        let (enc_hospital_personnel_pre_secret_key, hospital_personnel_pre_secret_key_nonce) =
            aes_encrypt_custom_key(sha_hash(pin.as_bytes()).as_slice(), &seed[0..32])
                .context(current_fn!())?;

        // Encrypt IOTA keypair
        let (enc_hospital_personnel_iota_key_pair, hospital_personnel_iota_key_pair_nonce) =
            aes_encrypt_custom_key(
                sha_hash(pin.as_bytes()).as_slice(),
                hospital_personnel_iota_key_pair
                    .encode()
                    .unwrap()
                    .as_bytes(),
            )
            .context(current_fn!())?;

        (
            private_administrative_data,
            private_administrative_metadata,
            public_administrative_data,
            enc_hospital_personnel_pre_secret_key,
            hospital_personnel_pre_secret_key_nonce,
            enc_hospital_personnel_iota_key_pair,
            hospital_personnel_iota_key_pair_nonce,
        )
    };

    let _ = state
        .move_call
        .signup(
            activation_key,
            hospital_personnnel_hospital_part_hash,
            hospital_personnnel_id_part_hash,
            serde_serialize_to_base64(&private_administrative_metadata).context(current_fn!())?,
            serde_serialize_to_base64(&public_administrative_data).context(current_fn!())?,
            hospital_personnel_iota_address,
            hospital_personnel_iota_key_pair,
        )
        .await
        .context(current_fn!())?;

    keys_entry.iota_address = Some(hospital_personnel_iota_address.to_string());
    keys_entry.iota_key_pair = Some(STANDARD.encode(enc_hospital_personnel_iota_key_pair));
    keys_entry.pre_secret_key = Some(STANDARD.encode(enc_hospital_personnel_pre_secret_key));
    keys_entry.pre_public_key =
        Some(serde_serialize_to_base64(&hospital_personnel_pre_public_key).context(current_fn!())?);
    keys_entry.pre_nonce = Some(STANDARD.encode(hospital_personnel_pre_secret_key_nonce));
    keys_entry.iota_nonce = Some(STANDARD.encode(hospital_personnel_iota_keypair_nonce));
    let keys_entry = serde_json::to_vec(&keys_entry).context(current_fn!())?;
    state
        .keys_entry
        .set_secret(&keys_entry)
        .context(current_fn!())?;

    state.administrative_data = Some(AdministrativeData {
        private: private_administrative_data,
        public: public_administrative_data,
    });

    // drop SignupState from state
    state.signup_state.seed_words = None;
    state.signup_state.pin = None;

    Ok(SuccessResponse {
        status: ResponseStatus::Success,
        data: (),
    })
}
