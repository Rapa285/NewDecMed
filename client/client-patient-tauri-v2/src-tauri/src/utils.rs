use anyhow::{anyhow, Context, Result};
use bip39::Mnemonic;
use chrono::{DateTime, Utc};
use image::{EncodableLayout, ImageReader};
use iota_sdk::{IotaClient, IotaClientBuilder};
use serde::{de::DeserializeOwned, Serialize};
use tauri_plugin_http::reqwest::{self, Client, IntoUrl, StatusCode};

use std::{fmt::Debug, io::Cursor, str::FromStr, time::SystemTime};

use aes_gcm::{aead::Aead, AeadCore, Aes256Gcm, KeyInit, Nonce};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Algorithm, Argon2, Params, PasswordHash, PasswordVerifier, Version,
};
use iota_json_rpc_types::{
    DevInspectResults, IotaObjectDataOptions, IotaTransactionBlockEffectsAPI,IotaTransactionBlockEffects,
};
use iota_keys::key_derive::derive_key_pair_from_path;
use iota_types::base_types::{IotaAddress, ObjectID, ObjectRef};
use iota_types::crypto::{EmptySignInfo, IotaKeyPair, SignatureScheme};
use iota_types::message_envelope::Envelope;
use iota_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_types::transaction::{
    CallArg, ObjectArg, ProgrammableTransaction, SenderSignedData, TransactionData,
    TransactionDataAPI,
};
use iota_types::{Identifier, TypeTag};
use rand::Rng;
use regex::Regex;
use serde_json::json;
use sha2::{Digest, Sha256};
use umbral_pre::{PublicKey, SecretKey, SecretKeyFactory};

use crate::{
    constants::IPFS_GATEWAY_BASE_URL,
    types::{ExecuteTxResponse, KeysEntry, ReserveGasResponse},
};
use crate::{
    constants::{GAS_STATION_BASE_URL, HASH_SALT, IOTA_URL},
    current_fn,
    als::{
        AuditEvent, AuditEventDetails, AuditOutcome, Event,
        AuditActionType, AuditActorType, AuditTargetObjectType, ALSClient
    },
    types::{AppState}
};
use base64::{engine::general_purpose::STANDARD, Engine as _};

pub async fn reserve_gas(
    state: &AppState,
    gas_budget: u64,
    reserve_duration_secs: u64,
) -> Result<(IotaAddress, u64, Vec<ObjectRef>)> {
    let req_client = reqwest::Client::new();
    let res = req_client
        .post(format!("{}/reserve_gas", GAS_STATION_BASE_URL))
        .bearer_auth("token")
        .json(&json!({
            "gas_budget": gas_budget,
        "reserve_duration_secs": reserve_duration_secs
        }))
        .send()
        .await
        .context(current_fn!())?;
    let res_body = res
        .json::<ReserveGasResponse>()
        .await
        .context(current_fn!())?;
    
    let (sponsor_account, reservation_id, gas_coins) = res_body
        .result
        .map(|result| {
            (
                result.sponsor_address,
                result.reservation_id,
                result
                    .gas_coins
                    .into_iter()
                    .map(|c| c.to_object_ref())
                    .collect::<Vec<_>>(),
            )
        })
        .context(current_fn!())?;
    
    // ── Audit: EV9 - Gas Sponsorship Request ───────────────────────────────────────────
    {
        // let state = state.lock().await;
        let keys_entry = parse_keys_entry(&state.keys_entry.get_secret().context(current_fn!())?)
            .context(current_fn!())?;

        // let hospital_personnel_iota_address =
        //         get_iota_address_from_keys_entry(&keys_entry).context(current_fn!())?;

        // let role = state
        //     .auth_state
        //     .role
        //     .clone()
        //     .ok_or(anyhow!("Role not found"))?;

        let event = Event {
            actor_id: "HospitalClient".to_string(),
            actor_type: AuditActorType::Client,
            target_object_type: AuditTargetObjectType::GasReservation,
            target_object: reservation_id.to_string(),
            outcome: AuditOutcome::Success,
            action_type: AuditActionType::Execute,
            details: AuditEventDetails::GasSponsorship {
                gas_budget_requested: gas_budget,
                reserve_duration_secs: reserve_duration_secs.clone(),
                reservation_id: Some(reservation_id),
                sponsor_address: Some(sponsor_account.to_string()),
                transaction_digest: None,
                gas_coin_object_ids: gas_coins
                    .iter()
                    .map(|obj_ref| obj_ref.0.to_string())
                    .collect(),
            },
        };
        let _ = ALSClient::send_event_from_state(&state, event, "create_capability");
    }
    // ──────────────────────────────────────────────────────────────────────────────────

    Ok((sponsor_account, reservation_id, gas_coins))
}

pub async fn execute_tx(
    state: &AppState,
    tx: Envelope<SenderSignedData, EmptySignInfo>,
    reservation_id: u64,
) -> Result<ExecuteTxResponse> {

    let signer_identity = tx.data().intent_message().value.sender().to_string();
    let payload_hash = tx.digest().to_string();
    let tx_data = &tx.data().intent_message().value;
    let sponsor_address = tx_data.gas_data().owner.to_string();

    // println!("tx data : {data:?}");
    let (tx_base_64, signature_base_64) = tx.to_tx_bytes_and_signatures();

    let req_client = reqwest::Client::new();
    let res = req_client
        .post(format!("{}/execute_tx", GAS_STATION_BASE_URL))
        .bearer_auth("token")
        .json(&json!({
            "reservation_id": reservation_id,
            "tx_bytes": tx_base_64.encoded(),
            "user_sig": signature_base_64[0].encoded()
        }))
        .send()
        .await
        .context(current_fn!())?;
    
    // ── Audit: EV8 - IOTA TX ───────────────────────────────────────────────────
    
    let ex_tx_res = res
        .json::<ExecuteTxResponse>()
        .await
        .context(current_fn!())?;
    
    // println!("ex tx res : {ex_tx_res:?}");

    let mut transaction_digest = String::from("unknown");
    let mut network_confirmation_status = String::from("unknown");
    let mut gas_used: Option<u64> = None;
    let mut is_success = AuditOutcome::Failure;

    if let Some(ref effects_enum) = ex_tx_res.effects {
        let IotaTransactionBlockEffects::V1(ref effects) = effects_enum;

        // Ambil Digest dari efek yang benar-benar terjadi
        transaction_digest = effects.transaction_digest.to_string();

        // Cek status kesuksesan transaksi
        if effects.status.is_ok() {
            network_confirmation_status = String::from("success");
            is_success = AuditOutcome::Success;
        } else {
            network_confirmation_status = String::from("failure");
            // Bisa juga ambil pesan error-nya jika ada
            // network_confirmation_status = format!("failure: {:?}", effects.status);
        }

        // Hitung total gas used (computation + storage - rebate)
        let gas = &effects.gas_used;
        let total_gas = (gas.computation_cost + gas.storage_cost).saturating_sub(gas.storage_rebate);
        gas_used = Some(total_gas);
    }

    let mut move_module = String::from("unknown");
    let mut move_function = String::from("unknown");

    // Lakukan pattern matching untuk mengekstrak Programmable Transaction
    if let iota_types::transaction::TransactionKind::ProgrammableTransaction(pt) = tx_data.kind() {
        // Karena dalam 1 transaksi bisa ada banyak perintah (commands),
        // kita loop untuk mencari perintah MoveCall.
        for command in &pt.commands {
            if let iota_types::transaction::Command::MoveCall(move_call) = command {
                move_module = move_call.module.to_string();
                move_function = move_call.function.to_string();
                
                // Jika kamu hanya butuh MoveCall pertama, kita bisa break di sini
                break; 
            }
        }
    }

    let event = Event {
        actor_id: signer_identity,
        actor_type: AuditActorType::Ministry,
        target_object_type: AuditTargetObjectType::Transaction,
        target_object: serde_json::to_string(tx_data).unwrap_or_else(|_| "Error serializing data".to_string()),
        outcome: is_success,
        action_type: AuditActionType::Execute,
        details: AuditEventDetails::IotaTransaction {
            transaction_digest: transaction_digest,
            payload_hash: payload_hash,
            move_function: move_function,
            move_module: move_module,
            network_confirmation_status: network_confirmation_status,
            gas_used: gas_used,
            sponsor_address: sponsor_address,
        },
    };

   let _ = ALSClient::send_event_from_state(&state, event,"iota_transaction");

    Ok(ex_tx_res)
}

pub fn parse_keys_entry(keys_entry: &Vec<u8>) -> Result<KeysEntry> {
    serde_json::from_slice(keys_entry).context(current_fn!())
}

pub fn generate_64_bytes_seed() -> [u8; 64] {
    let mut rng = rand::thread_rng();
    let mut random_seed = [0u8; 64];
    rng.fill(&mut random_seed);

    random_seed
}

pub fn generate_iota_keys_ed(seed: &[u8]) -> Result<(IotaAddress, IotaKeyPair)> {
    Ok(derive_key_pair_from_path(
        &seed,
        Some(bip32::DerivationPath::from_str("m/44'/4218'/0'/0'/0'").unwrap()),
        &SignatureScheme::ED25519,
    )
    .context(current_fn!())?)
}

pub fn construct_pt(
    function_name: String,
    package: ObjectID,
    module: Identifier,
    type_arguments: Vec<TypeTag>,
    call_args: Vec<CallArg>,
) -> Result<ProgrammableTransaction> {
    let mut builder = ProgrammableTransactionBuilder::new();
    let function = Identifier::from_str(function_name.as_str()).context(current_fn!())?;

    builder
        .move_call(package, module, function, type_arguments, call_args)
        .context(current_fn!())?;

    Ok(builder.finish())
}

pub fn construct_sponsored_tx_data(
    sender: IotaAddress,
    gas_payment: Vec<ObjectRef>,
    pt: ProgrammableTransaction,
    gas_budget: u64,
    gas_price: u64,
    sponsor_address: IotaAddress,
) -> TransactionData {
    let mut tx_data =
        TransactionData::new_programmable(sender, gas_payment.clone(), pt, gas_budget, gas_price);

    tx_data.gas_data_mut().payment = gas_payment;
    tx_data.gas_data_mut().owner = sponsor_address;

    tx_data
}

pub async fn get_ref_gas_price(iota_client: &IotaClient) -> Result<u64> {
    Ok((*iota_client)
        .governance_api()
        .get_reference_gas_price()
        .await
        .context(current_fn!())?)
}

pub async fn _construct_capability_call_arg(
    iota_client: &IotaClient,
    capability_id: ObjectID,
) -> Result<CallArg> {
    let cap_object = (*iota_client)
        .read_api()
        .get_object_with_options(
            capability_id,
            IotaObjectDataOptions {
                ..Default::default()
            },
        )
        .await
        .context(current_fn!())?;

    let cap_object_arg = ObjectArg::ImmOrOwnedObject((
        cap_object.data.clone().unwrap().object_id,
        cap_object.data.clone().unwrap().version,
        cap_object.data.unwrap().digest,
    ));

    Ok(CallArg::Object(cap_object_arg))
}

pub fn construct_shared_object_call_arg(id: ObjectID, version: u64, mutable: bool) -> CallArg {
    let activation_key_table_arg = ObjectArg::SharedObject {
        id,
        initial_shared_version: version.into(),
        mutable,
    };

    CallArg::Object(activation_key_table_arg)
}

pub async fn move_call_read_only(
    sender: IotaAddress,
    iota_client: &IotaClient,
    pt: ProgrammableTransaction,
) -> Result<DevInspectResults> {
    Ok((*iota_client)
        .read_api()
        .dev_inspect_transaction_block(
            sender,
            iota_types::transaction::TransactionKind::ProgrammableTransaction(pt),
            None,
            None,
            None,
        )
        .await
        .context(current_fn!())?)
}

pub fn argon_hash(password: String) -> Result<String> {
    let salt = SaltString::from_b64(HASH_SALT)
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;
    let argon2 = Argon2::new_with_secret(
        HASH_SALT.as_bytes(),
        Algorithm::Argon2id,
        Version::V0x13,
        Params::DEFAULT,
    )
    .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;

    let hash = argon2
        .hash_password(password.as_str().as_bytes(), &salt)
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?
        .to_string();

    Ok(hex::encode(hash))
}

pub fn _argon_verify(hash: String, password: String) -> Result<bool> {
    let argon2 = Argon2::new_with_secret(
        HASH_SALT.as_bytes(),
        Algorithm::Argon2id,
        Version::V0x13,
        Params::DEFAULT,
    )
    .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;
    let hash = PasswordHash::new(hash.as_str())
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;

    Ok(argon2.verify_password(password.as_bytes(), &hash).is_ok())
}

/**
* output:
* key: 32 bytes
* nonce: 12 bytes
* return: `(ciphertext, key, nonce)`
*/
pub fn aes_encrypt(data: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let key = Aes256Gcm::generate_key(aes_gcm::aead::OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut aes_gcm::aead::OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, data)
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;

    Ok((ciphertext, key.to_vec(), nonce.to_vec()))
}

pub fn aes_encrypt_custom_key(key: &[u8], data: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    let cipher = Aes256Gcm::new_from_slice(key).context(current_fn!())?;
    let nonce = Aes256Gcm::generate_nonce(&mut aes_gcm::aead::OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, data)
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;

    Ok((ciphertext, nonce.to_vec()))
}

pub fn aes_decrypt(ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key).context(current_fn!())?;
    let nonce = Nonce::from_slice(nonce);

    let original = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;

    Ok(original)
}

pub fn sha_hash(data: &[u8]) -> Vec<u8> {
    let hash = Sha256::digest(data);
    hash.to_vec()
}

pub fn compute_pre_keys(seed: &[u8]) -> Result<(SecretKey, PublicKey)> {
    let secret_key = SecretKeyFactory::from_secure_randomness(seed)
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?
        .make_key(seed);
    let public_key = secret_key.public_key();

    Ok((secret_key, public_key))
}

pub fn parse_move_read_only_result<T: DeserializeOwned>(
    val: DevInspectResults,
    index: usize,
) -> Result<T> {
    let res = val.results.context(current_fn!())?[0].return_values[index]
        .0
        .to_vec();

    Ok(bcs::from_bytes::<T>(&res).context(current_fn!())?)
}

pub async fn get_iota_client() -> Result<IotaClient> {
    Ok(IotaClientBuilder::default()
        .build(IOTA_URL)
        .await
        .context(current_fn!())?)
}

pub fn handle_error_move_call_read_only(response: DevInspectResults) -> Result<()> {
    if response.error.is_some() {
        return Err(anyhow!(response.error.unwrap()).context(current_fn!()));
    }

    if response.effects.status().is_err() {
        return Err(anyhow!(response.effects.status().to_string()).context(current_fn!()));
    }

    Ok(())
}

pub fn handle_error_execute_tx(response: ExecuteTxResponse) -> Result<()> {
    if response.error.is_some() {
        return Err(anyhow!(response.error.unwrap()).context(current_fn!()));
    }

    if response.effects.is_some() && response.effects.as_ref().unwrap().status().is_err() {
        return Err(anyhow!(response.effects.unwrap().status().to_string()).context(current_fn!()));
    }

    Ok(())
}

/**
 * Return: `(metadata, content)`
 */
pub fn process_qr_image(qr_image_bytes: &[u8]) -> Result<(rqrr::MetaData, String)> {
    let reader = ImageReader::new(Cursor::new(qr_image_bytes))
        .with_guessed_format()
        .context(current_fn!())?;
    let image = reader.decode().context(current_fn!())?;
    let image = image.to_luma8();
    let mut prepared_image = rqrr::PreparedImage::prepare(image);
    let grids = prepared_image.detect_grids();

    Ok(grids[0].decode().context(current_fn!())?)
}

pub fn validate_by_regex(value: &str, regex: &str) -> Result<bool> {
    let re = Regex::new(regex).context(current_fn!())?;
    Ok(re.is_match(value))
}

pub fn decode_hospital_personnel_qr(content: String) -> Result<(IotaAddress, PublicKey)> {
    let content: Vec<&str> = content.split("@").collect();

    if content.len() != 2 {
        return Err(anyhow!(
            "Invalid content length, expected 2 found {}",
            content.len()
        ))
        .context(current_fn!());
    }

    let iota_address = IotaAddress::from_str(content[0]).context(current_fn!())?;
    let pre_public_key =
        serde_deserialize_from_base64(content[1].to_string()).context(current_fn!())?;

    Ok((iota_address, pre_public_key))
}

pub async fn do_http_post_json_request<P, T, E>(
    access_token: Option<String>,
    endpoint: &str,
    payload: &P,
    req_client: &Client,
    success_status_code: StatusCode,
) -> Result<T>
where
    P: Serialize,
    E: DeserializeOwned + Debug,
    T: DeserializeOwned,
{
    let mut res = req_client.post(endpoint).json(payload);
    if access_token.is_some() {
        res = res.bearer_auth(access_token.unwrap());
    }
    let res = res.send().await.context(current_fn!())?;

    let res_status = res.status();
    let res_body = res.bytes().await.context(current_fn!())?;

    if res_status != success_status_code {
        let error: E = serde_json::from_slice(res_body.as_bytes()).context(current_fn!())?;
        return Err(anyhow!(format!("{:#?}", error)).context(current_fn!()));
    }

    let res_body: T = serde_json::from_slice(res_body.as_bytes()).context(current_fn!())?;

    Ok(res_body)
}

pub fn sys_time_to_iso(system_time: SystemTime) -> String {
    let iso: DateTime<Utc> = system_time.into();
    iso.to_rfc3339()
}

pub fn get_iota_address_from_keys_entry(keys_entry: &KeysEntry) -> Result<IotaAddress> {
    let iota_address = keys_entry
        .iota_address
        .as_ref()
        .ok_or(anyhow!("IOTA Address not found on keys entry").context(current_fn!()))?;

    Ok(IotaAddress::from_str(&iota_address).context(current_fn!())?)
}

pub fn get_iota_key_pair_from_keys_entry(
    keys_entry: &KeysEntry,
    pin: String,
) -> Result<IotaKeyPair> {
    let iota_key_pair = STANDARD
        .decode(
            keys_entry
                .iota_key_pair
                .as_ref()
                .ok_or(anyhow!("IOTA Key Pair not found on keys entry").context(current_fn!()))?,
        )
        .context(current_fn!())?;
    let iota_key_pair_nonce =
        STANDARD
            .decode(keys_entry.iota_nonce.as_ref().ok_or(
                anyhow!("IOTA Key Pair Nonce not found on keys entry").context(current_fn!()),
            )?)
            .context(current_fn!())?;
    let iota_key_pair = aes_decrypt(
        &iota_key_pair,
        &sha_hash(pin.as_bytes()),
        &iota_key_pair_nonce,
    )?;
    let iota_key_pair = String::from_utf8(iota_key_pair).context(current_fn!())?;
    let iota_key_pair = IotaKeyPair::decode(&iota_key_pair)
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;

    Ok(iota_key_pair)
}

pub fn _get_pre_public_key_from_keys_entry(keys_entry: &KeysEntry) -> Result<PublicKey> {
    serde_deserialize_from_base64(
        keys_entry
            .pre_public_key
            .clone()
            .ok_or(anyhow!("PRE Public Key not found on keys entry").context(current_fn!()))?,
    )
    .context(current_fn!())
}

pub fn get_pre_keys_from_keys_entry(
    keys_entry: &KeysEntry,
    pin: String,
) -> Result<(SecretKey, PublicKey)> {
    let pre_seed = STANDARD
        .decode(
            keys_entry
                .pre_secret_key
                .as_ref()
                .ok_or(anyhow!("PRE Seed not found on keys entry").context(current_fn!()))?,
        )
        .context(current_fn!())?;

    let pre_seed_nonce = STANDARD
        .decode(
            keys_entry
                .pre_nonce
                .as_ref()
                .ok_or(anyhow!("PRE Seed nonce not found on keys entry").context(current_fn!()))?,
        )
        .context(current_fn!())?;

    let pre_seed = aes_decrypt(&pre_seed, &sha_hash(pin.as_bytes()), &pre_seed_nonce)?;

    let pre_secret_key = SecretKeyFactory::from_secure_randomness(&pre_seed)
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?
        .make_key(&pre_seed);
    let pre_public_key = pre_secret_key.public_key();

    Ok((pre_secret_key, pre_public_key))
}

pub fn serde_serialize_to_base64<T>(val: &T) -> Result<String>
where
    T: Serialize,
{
    let ser_val = serde_json::to_vec(val).context(current_fn!())?;
    Ok(STANDARD.encode(ser_val))
}

pub fn serde_deserialize_from_base64<T>(val: String) -> Result<T>
where
    T: DeserializeOwned,
{
    let val = STANDARD.decode(val).context(current_fn!())?;
    let ori_val: T = serde_json::from_slice(&val).context(current_fn!())?;

    Ok(ori_val)
}

pub fn compute_seed_from_seed_words(seed_words: &str, passphrase: &str) -> Result<[u8; 64]> {
    let mnemonic = Mnemonic::from_str(seed_words).context(current_fn!())?;
    Ok(mnemonic.to_seed_normalized(passphrase))
}

pub async fn do_http_get_request<T, E, U>(
    req_client: &Client,
    success_status_code: StatusCode,
    url: U,
) -> Result<T>
where
    T: DeserializeOwned + From<String>,
    E: DeserializeOwned + Debug,
    U: IntoUrl,
{
    let res = req_client.get(url).send().await.context(current_fn!())?;
    let res_status = res.status();
    let content_type = res
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .ok_or(anyhow!("Failed to get content type from header").context(current_fn!()))?
        .to_string();
    let res_body = res.bytes().await.context(current_fn!())?;

    if res_status != success_status_code {
        let error: E = serde_json::from_slice(&res_body.to_vec()).context(current_fn!())?;

        return Err(anyhow!(format!("{:#?}", error)).context(current_fn!()));
    }

    match content_type.as_str() {
        "application/json" => {
            Ok(serde_json::from_slice(&res_body.to_vec()).context(current_fn!())?)
        }
        "text/plain; charset=utf-8" => Ok(T::from(
            String::from_utf8(res_body.to_vec()).context(current_fn!())?,
        )),
        _ => {
            return Err(
                anyhow!(format!("Unknown content-type: {}", content_type)).context(current_fn!())
            )
        }
    }
}

pub async fn get_data_ipfs(cid: String) -> Result<String> {
    let req_client = reqwest::Client::new();
    let content = do_http_get_request::<String, String, _>(
        &req_client,
        StatusCode::OK,
        format!("{}/ipfs/{}", IPFS_GATEWAY_BASE_URL, cid),
    )
    .await
    .context(current_fn!())?;

    Ok(content)
}
