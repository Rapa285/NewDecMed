use anyhow::{anyhow, Context};
use tauri::{async_runtime::Mutex, State};

use crate::{
    current_fn,
    hospital_error::HospitalError,
    types::{
        AdministrativeData, AppState, PrivateAdministrativeData, PublicAdministrativeData,
        ResponseStatus, SuccessResponse,
    },
    utils::{
        aes_encrypt_custom_key, compute_pre_keys, compute_seed_from_seed_words,
        encode_activation_key_from_keys_entry, generate_iota_keys_ed, parse_keys_entry,
        serde_serialize_to_base64, sha_hash,get_iota_key_pair_from_keys_entry,get_iota_address_from_keys_entry,
    },
    als::{
        AuditEvent, AuditEventDetails, AuditOutcome, Event,
        AuditActionType, AuditActorType, AuditTargetObjectType, ALSClient
    },

};
use base64::{engine::general_purpose::STANDARD, Engine as _};

#[tauri::command]
pub async fn signin(
    state: State<'_, Mutex<AppState>>,
    seed_words: String,
) -> Result<SuccessResponse<()>, HospitalError> {
    let mut state = state.lock().await;
    let mut keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
        .context(current_fn!())?;

    let (
        pin,
        seed,
        activation_key,
        hospital_personnel_iota_address,
        hospital_personnel_iota_key_pair,
        hospital_personnel_pre_public_key,
    ) = {
        let pin = state
            .signin_state
            .pin
            .clone()
            .ok_or(anyhow!("PIN not found on signin state").context(current_fn!()))?;
        let seed = compute_seed_from_seed_words(
            &seed_words,
            &keys_entry
                .id
                .as_ref()
                .ok_or(anyhow!("Id not found on keys entry").context(current_fn!()))?,
        )?;
        let activation_key =
            encode_activation_key_from_keys_entry(&keys_entry).context(current_fn!())?;
        let (hospital_personnel_iota_address, hospital_personnel_iota_key_pair) =
            generate_iota_keys_ed(&seed).context(current_fn!())?;
        let (_, hospital_personnel_pre_public_key) =
            compute_pre_keys(&seed[0..32]).context(current_fn!())?;

        (
            pin,
            seed,
            activation_key,
            hospital_personnel_iota_address,
            hospital_personnel_iota_key_pair,
            hospital_personnel_pre_public_key,
        )
    };

    let (enc_hospital_personnel_pre_secret_key, hospital_personnel_pre_secret_key_nonce) =
        aes_encrypt_custom_key(&sha_hash(pin.as_bytes()), &seed[0..32]).context(current_fn!())?;
    let (enc_hospital_personnel_iota_key_pair, hospital_personnel_iota_key_pair_nonce) =
        aes_encrypt_custom_key(
            &sha_hash(pin.as_bytes()),
            hospital_personnel_iota_key_pair
                .encode()
                .unwrap()
                .as_bytes(),
        )
        .context(current_fn!())?;

    let is_registered: bool = state
        .move_call
        .is_account_registered(activation_key, hospital_personnel_iota_address.clone())
        .await
        .context(current_fn!())?;

    if !is_registered {
        // ── Audit: EV1 - Authentication (Failure: Account Not Found) ─────────────────────
        {

            let event = Event {
                actor_id: "Unknown User".to_string(),
                actor_type: AuditActorType::Unknown,
                target_object_type: AuditTargetObjectType::Account,
                target_object: hospital_personnel_iota_address.to_string().clone(),
                outcome: AuditOutcome::Failure,
                action_type: AuditActionType::SignIn,
                details: AuditEventDetails::Authentication {
                    auth_method: "pin".to_string(),
                    role: "Unregistered".to_string(),
                    failure_reason: Some("Account not registered".to_string()),
                },
            };
            let _ = ALSClient::send_event_from_state(&state, event,"signin");
        }
        // ──────────────────────────────────────────────────────────────────────────────────

        return Err(HospitalError::Anyhow(anyhow!("Account not found")));
    }

    keys_entry.iota_address = Some(hospital_personnel_iota_address.to_string());
    keys_entry.iota_key_pair = Some(STANDARD.encode(enc_hospital_personnel_iota_key_pair));
    keys_entry.pre_secret_key = Some(STANDARD.encode(enc_hospital_personnel_pre_secret_key));
    keys_entry.pre_public_key =
        Some(serde_serialize_to_base64(&hospital_personnel_pre_public_key).context(current_fn!())?);
    keys_entry.pre_nonce = Some(STANDARD.encode(hospital_personnel_pre_secret_key_nonce));
    keys_entry.iota_nonce = Some(STANDARD.encode(hospital_personnel_iota_key_pair_nonce));
    state
        .keys_entry
        .set_secret(&serde_json::to_vec(&keys_entry).context(current_fn!())?)
        .context(current_fn!())?;

    if state.administrative_data.is_none() {
        state.administrative_data = Some(AdministrativeData {
            private: PrivateAdministrativeData {
                id: keys_entry.id.clone().unwrap(),
            },
            public: PublicAdministrativeData { name: None },
        })
    }

    let actor_key_pair = get_iota_key_pair_from_keys_entry(&keys_entry, pin)?;
    let actor_address  = get_iota_address_from_keys_entry(&keys_entry)?;

    // drop SigninState form state
    state.signin_state.pin = None;

    // ── Audit: EV1 - Authentication (Success) ─────────────────────────────────────────
    {
        let role = state
            .auth_state
            .role
            .clone()
            .ok_or(anyhow!("Role not found"))?;

        let event = Event {
            actor_id: hospital_personnel_iota_address.to_string().clone(),
            actor_type: AuditActorType::from(role),
            target_object_type: AuditTargetObjectType::Account,
            target_object: hospital_personnel_iota_address.to_string().clone(),
            outcome: AuditOutcome::Success,
            action_type: AuditActionType::SignIn,
            details: AuditEventDetails::Authentication {
                auth_method: "pin".to_string(),
                role: format!("{:?}", role),
                failure_reason: Some("Account not registered".to_string()),
            },
        };
        let _ = ALSClient::send_event_from_state(&state, event,"signin");
    }
    // ──────────────────────────────────────────────────────────────────────────────────

    Ok(SuccessResponse {
        status: ResponseStatus::Success,
        data: (),
    })
}
