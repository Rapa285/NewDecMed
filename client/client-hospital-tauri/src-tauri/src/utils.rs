use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use aes_gcm::{aead::Aead, AeadCore, Aes256Gcm, KeyInit, Nonce};
use anyhow::{anyhow, Context};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Algorithm, Argon2, Params, PasswordHash, PasswordVerifier, Version,
};
use bip39::Mnemonic;
use iota_json_rpc_types::{
    DevInspectResults, IotaObjectDataOptions, IotaTransactionBlockEffectsAPI,
};
use iota_keys::key_derive::derive_key_pair_from_path;
use iota_sdk::{IotaClient, IotaClientBuilder};
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
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use tauri::http::StatusCode;
use tauri_plugin_http::reqwest::{self, Client, IntoUrl};
use umbral_pre::{PublicKey, SecretKey, SecretKeyFactory};

use crate::constants::{GAS_STATION_BASE_URL, HASH_SALT};
use crate::{
    constants::{IOTA_URL, _IPFS_GATEWAY_BASE_URL},
    current_fn,
    hospital_error::HospitalError,
    types::{ExecuteTxResponse, KeysEntry, ReserveGasResponse},
};
use base64::{engine::general_purpose::STANDARD, Engine as _};

pub async fn reserve_gas(
    gas_budget: u64,
    reserve_duration_secs: u64,
) -> Result<(IotaAddress, u64, Vec<ObjectRef>), HospitalError> {
    let req_client = reqwest::Client::new();
    let res = req_client
        .post(format!("{GAS_STATION_BASE_URL}/reserve_gas"))
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
    // println!("{:#?}", res_body);
    Ok(res_body
        .result
        .map(|result| {
            (
                result.sponsor_address,
                result.reservation_id,
                result
                    .gas_coins
                    .into_iter()
                    .map(|c| c.to_object_ref())
                    .collect(),
            )
        })
        .ok_or(anyhow!("Failed to map response body").context(current_fn!()))?)
}

pub async fn execute_tx(
    tx: Envelope<SenderSignedData, EmptySignInfo>,
    reservation_id: u64,
) -> Result<ExecuteTxResponse, HospitalError> {
    let (tx_base_64, signature_base_64) = tx.to_tx_bytes_and_signatures();

    let req_client = reqwest::Client::new();
    let res = req_client
        .post(format!("{GAS_STATION_BASE_URL}/execute_tx"))
        .bearer_auth("token")
        .json(&json!({
            "reservation_id": reservation_id,
            "tx_bytes": tx_base_64.encoded(),
            "user_sig": signature_base_64[0].encoded()
        }))
        .send()
        .await
        .context(current_fn!())?;

    Ok(res
        .json::<ExecuteTxResponse>()
        .await
        .context(current_fn!())?)
}

pub fn parse_keys_entry(keys_entry: &Vec<u8>) -> Result<KeysEntry, HospitalError> {
    Ok(serde_json::from_slice(keys_entry).context(current_fn!())?)
}

pub fn generate_64_bytes_seed() -> [u8; 64] {
    let mut rng = rand::rng();
    let mut random_seed = [0u8; 64];
    rng.fill(&mut random_seed);

    random_seed
}

pub fn generate_iota_keys_ed(seed: &[u8]) -> Result<(IotaAddress, IotaKeyPair), HospitalError> {
    Ok(derive_key_pair_from_path(
        &seed,
        Some(bip32::DerivationPath::from_str("m/44'/4218'/0'/0'/0'").context(current_fn!())?),
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
) -> Result<ProgrammableTransaction, HospitalError> {
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

pub async fn get_ref_gas_price(iota_client: &IotaClient) -> Result<u64, HospitalError> {
    Ok((*iota_client)
        .governance_api()
        .get_reference_gas_price()
        .await
        .context(current_fn!())?)
}

pub async fn construct_capability_call_arg(
    iota_client: &IotaClient,
    capability_id: ObjectID,
) -> Result<CallArg, HospitalError> {
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

    let cap_object_data = cap_object
        .data
        .ok_or(anyhow!("Cap object data not found").context(current_fn!()))?;

    let cap_object_arg = ObjectArg::ImmOrOwnedObject((
        cap_object_data.object_id,
        cap_object_data.version,
        cap_object_data.digest,
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
) -> Result<DevInspectResults, HospitalError> {
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

pub fn argon_hash(password: String) -> Result<String, HospitalError> {
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

pub fn _argon_verify(hash: String, password: String) -> Result<bool, HospitalError> {
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
* return: (ciphertext, key, nonce)
*/
pub fn aes_encrypt(data: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), HospitalError> {
    let key = Aes256Gcm::generate_key(aes_gcm::aead::OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut aes_gcm::aead::OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, data)
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;

    Ok((ciphertext, key.to_vec(), nonce.to_vec()))
}

pub fn aes_encrypt_custom_key(
    key: &[u8],
    data: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), HospitalError> {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let nonce = Aes256Gcm::generate_nonce(&mut aes_gcm::aead::OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, data)
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?;

    Ok((ciphertext, nonce.to_vec()))
}

pub fn aes_decrypt(ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>, HospitalError> {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
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

pub fn _sha_hash_to_hex(data: &[u8]) -> String {
    let hash = Sha256::digest(data);
    hex::encode(hash)
}

pub fn validate_by_regex(value: &str, regex: &str) -> Result<bool, HospitalError> {
    let re = Regex::new(regex).context(current_fn!())?;
    Ok(re.is_match(value))
}

pub fn compute_pre_keys(seed: &[u8]) -> Result<(SecretKey, PublicKey), HospitalError> {
    let secret_key = SecretKeyFactory::from_secure_randomness(seed)
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?
        .make_key(seed);
    let public_key = secret_key.public_key();

    Ok((secret_key, public_key))
}

pub fn parse_move_read_only_result<T: DeserializeOwned>(
    val: DevInspectResults,
    index: usize,
) -> Result<T, HospitalError> {
    let res = val.results.context(current_fn!())?[0].return_values[index]
        .0
        .to_vec();

    Ok(bcs::from_bytes::<T>(&res).context(current_fn!())?)
}

pub async fn get_iota_client() -> Result<IotaClient, HospitalError> {
    Ok(IotaClientBuilder::default()
        .build(IOTA_URL)
        .await
        .context(current_fn!())?)
}

pub fn handle_error_move_call_read_only(response: DevInspectResults) -> Result<(), HospitalError> {
    if response.error.is_some() {
        return Err(HospitalError::Anyhow(
            anyhow!(response.error.unwrap()).context(current_fn!()),
        ));
    }

    if response.effects.status().is_err() {
        return Err(HospitalError::Anyhow(
            anyhow!(response.effects.status().to_string()).context(current_fn!()),
        ));
    }

    Ok(())
}

/**
 * Return: `(id_part, hospital_part)`
 */
pub fn _decode_hospital_personnel_id(id: String) -> Result<(String, String), HospitalError> {
    let id: Vec<&str> = id.split("@").collect();

    if id.len() != 2 {
        return Err(HospitalError::Anyhow(
            anyhow!("Invalid id").context(current_fn!()),
        ));
    }

    Ok((id[0].to_string(), id[1].to_string()))
}

pub fn handle_error_execute_tx(response: ExecuteTxResponse) -> Result<(), HospitalError> {
    if response.error.is_some() {
        return Err(HospitalError::Anyhow(
            anyhow!(response.error.unwrap()).context(current_fn!()),
        ));
    }

    if response.effects.is_some() && response.effects.as_ref().unwrap().status().is_err() {
        return Err(HospitalError::Anyhow(
            anyhow!(response.effects.unwrap().status().to_string()).context(current_fn!()),
        ));
    }

    Ok(())
}

pub fn _decode_hospital_personnel_qr(
    content: String,
) -> Result<(IotaAddress, PublicKey), HospitalError> {
    let content: Vec<&str> = content.split("@").collect();

    if content.len() != 2 {
        return Err(HospitalError::Anyhow(
            anyhow!("Invalid content length, expected 2 found {}", content.len())
                .context(current_fn!()),
        ));
    }

    let iota_address = IotaAddress::from_str(content[0]).context(current_fn!())?;
    let pre_public_key =
        serde_deserialize_from_base64(content[1].to_string()).context(current_fn!())?;

    Ok((iota_address, pre_public_key))
}

pub async fn do_http_post_request_json<P, T, E>(
    access_token: Option<String>,
    endpoint: &str,
    payload: &P,
    req_client: &Client,
    success_status_code: StatusCode,
) -> Result<T, HospitalError>
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
        let error: E = serde_json::from_slice(&res_body.to_vec()).context(current_fn!())?;
        return Err(HospitalError::Anyhow(
            anyhow!(format!("{:#?}", error)).context(current_fn!()),
        ));
    }

    let res_body: T = serde_json::from_slice(&res_body.to_vec()).context(current_fn!())?;

    Ok(res_body)
}

pub async fn do_http_put_request_json<P, T, E>(
    access_token: Option<String>,
    endpoint: &str,
    payload: &P,
    req_client: &Client,
    success_status_code: StatusCode,
) -> Result<T, HospitalError>
where
    P: Serialize,
    E: DeserializeOwned + Debug,
    T: DeserializeOwned,
{
    let mut res = req_client.put(endpoint).json(payload);
    if access_token.is_some() {
        res = res.bearer_auth(access_token.unwrap());
    }
    let res = res.send().await.context(current_fn!())?;

    let res_status = res.status();
    let res_body = res.bytes().await.context(current_fn!())?;

    if res_status != success_status_code {
        let error: E = serde_json::from_slice(&res_body.to_vec()).context(current_fn!())?;
        return Err(HospitalError::Anyhow(
            anyhow!(format!("{:#?}", error)).context(current_fn!()),
        ));
    }

    let res_body: T = serde_json::from_slice(&res_body.to_vec()).context(current_fn!())?;

    Ok(res_body)
}

pub fn get_iota_address_from_keys_entry(
    keys_entry: &KeysEntry,
) -> Result<IotaAddress, HospitalError> {
    let iota_address = keys_entry
        .iota_address
        .as_ref()
        .ok_or(anyhow!("IOTA Address not found on keys entry").context(current_fn!()))?;

    Ok(IotaAddress::from_str(&iota_address).context(current_fn!())?)
}

pub fn get_iota_key_pair_from_keys_entry(
    keys_entry: &KeysEntry,
    pin: String,
) -> Result<IotaKeyPair, HospitalError> {
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

pub fn _get_pre_public_key_from_keys_entry(
    keys_entry: &KeysEntry,
) -> Result<PublicKey, HospitalError> {
    Ok(serde_deserialize_from_base64(
        keys_entry
            .pre_public_key
            .clone()
            .ok_or(anyhow!("PRE Public Key not found on keys entry").context(current_fn!()))?,
    )
    .context(current_fn!())?)
}

pub fn get_global_admin_iota_address_from_keys_entry(
    keys_entry: &KeysEntry,
) -> Result<IotaAddress, HospitalError> {
    Ok(
        IotaAddress::from_str(&keys_entry.admin_address.as_ref().ok_or(
            anyhow!("Global admin iota address not found on keys entry").context(current_fn!()),
        )?)
        .context(current_fn!())?,
    )
}

pub fn get_global_admin_iota_key_pair_from_keys_entry(
    keys_entry: &KeysEntry,
) -> Result<IotaKeyPair, HospitalError> {
    Ok(
        IotaKeyPair::decode(&keys_entry.admin_secret_key.as_ref().ok_or(
            anyhow!("Global admin iota key pair not found on keys entry").context(current_fn!()),
        )?)
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?,
    )
}

pub fn get_pre_keys_from_keys_entry(
    keys_entry: &KeysEntry,
    pin: String,
) -> Result<(SecretKey, PublicKey), HospitalError> {
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

pub fn serde_serialize_to_base64<T>(val: &T) -> Result<String, HospitalError>
where
    T: Serialize,
{
    let ser_val = serde_json::to_vec(val).context(current_fn!())?;
    Ok(STANDARD.encode(ser_val))
}

pub fn serde_deserialize_from_base64<T>(val: String) -> Result<T, HospitalError>
where
    T: DeserializeOwned,
{
    let val = STANDARD.decode(val).context(current_fn!())?;
    let ori_val: T = serde_json::from_slice(&val).context(current_fn!())?;

    Ok(ori_val)
}

pub fn compute_seed_from_seed_words(
    seed_words: &str,
    passphrase: &str,
) -> Result<[u8; 64], HospitalError> {
    let mnemonic = Mnemonic::from_str(seed_words).context(current_fn!())?;
    Ok(mnemonic.to_seed_normalized(passphrase))
}

pub async fn do_http_get_request_json<T, E, U>(
    access_token: Option<String>,
    req_client: &Client,
    success_status_code: StatusCode,
    url: U,
) -> Result<T, HospitalError>
where
    T: DeserializeOwned,
    E: DeserializeOwned + Debug,
    U: IntoUrl,
{
    let mut res = req_client.get(url);
    if access_token.is_some() {
        res = res.bearer_auth(access_token.unwrap());
    }
    let res = res.send().await.context(current_fn!())?;

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

        return Err(HospitalError::Anyhow(
            anyhow!(format!("{:#?}", error)).context(current_fn!()),
        ));
    }

    match content_type.as_str() {
        "application/json" => {
            Ok(serde_json::from_slice(&res_body.to_vec()).context(current_fn!())?)
        }
        _ => {
            return Err(HospitalError::Anyhow(
                anyhow!(format!("Unknown content-type: {}", content_type)).context(current_fn!()),
            ))
        }
    }
}

pub async fn _do_http_get_request_text<T, E, U>(
    access_token: Option<String>,
    req_client: &Client,
    success_status_code: StatusCode,
    url: U,
) -> Result<T, HospitalError>
where
    T: FromStr,
    T::Err: Display,
    E: DeserializeOwned + Debug,
    U: IntoUrl,
{
    let mut res = req_client.get(url);
    if access_token.is_some() {
        res = res.bearer_auth(access_token.unwrap());
    }
    let res = res.send().await.context(current_fn!())?;
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

        return Err(HospitalError::Anyhow(
            anyhow!(format!("{:#?}", error)).context(current_fn!()),
        ));
    }

    match content_type.as_str() {
        "text/plain; charset=utf-8" => Ok(T::from_str(
            &String::from_utf8(res_body.to_vec()).context(current_fn!())?,
        )
        .map_err(|e| anyhow!(e.to_string()))?),
        _ => {
            return Err(HospitalError::Anyhow(
                anyhow!(format!("Unknown content-type: {}", content_type)).context(current_fn!()),
            ))
        }
    }
}

pub async fn _get_data_ipfs(cid: String) -> Result<String, HospitalError> {
    let req_client = reqwest::Client::new();
    let content = _do_http_get_request_text::<String, String, _>(
        None,
        &req_client,
        StatusCode::OK,
        format!("{}/ipfs/{}", _IPFS_GATEWAY_BASE_URL, cid),
    )
    .await
    .context(current_fn!())?;

    Ok(content)
}

/**
 * return: `(id_part,hospital_part)`
 */
pub fn decode_hospital_personnel_id(id: String) -> Result<(String, String), HospitalError> {
    let id: Vec<&str> = id.split("@").collect();

    if id.len() != 2 {
        return Err(HospitalError::Anyhow(
            anyhow!(format!("Invalid id length, expected 2, found {}", id.len()))
                .context(current_fn!()),
        ));
    }

    Ok((id[0].to_string(), id[1].to_string()))
}

/**
 * return: `(id_part_hash,hospital_part_hash)`
 */
pub fn decode_hospital_personnel_id_to_argon(
    id: String,
) -> Result<(String, String), HospitalError> {
    let id: Vec<&str> = id.split("@").collect();

    if id.len() != 2 {
        return Err(HospitalError::Anyhow(
            anyhow!(format!("Invalid id length, expected 2, found {}", id.len()))
                .context(current_fn!()),
        ));
    }

    let id_part = argon_hash(id[0].to_string()).context(current_fn!())?;
    let hospital_part = argon_hash(id[1].to_string()).context(current_fn!())?;

    Ok((id_part, hospital_part))
}

/// ## Params:
/// - `activation_key`: raw_uuid_v4
/// - `id`: raw_id
pub fn encode_activation_key(activation_key: String, id: String) -> Result<String, HospitalError> {
    let activation_key = format!("{}@{}", activation_key, id);

    Ok(STANDARD.encode(argon_hash(activation_key).context(current_fn!())?))
}

/// ## Params:
/// - `activation_key`: raw_uuid_v4
/// - `id`: raw_id
pub fn encode_activation_key_from_keys_entry(
    keys_entry: &KeysEntry,
) -> Result<String, HospitalError> {
    let activation_key = keys_entry
        .activation_key
        .clone()
        .ok_or(anyhow!("Activation key not found on keys entry").context(current_fn!()))?;
    let id = keys_entry
        .id
        .clone()
        .ok_or(anyhow!("Id not found on keys entry").context(current_fn!()))?;

    encode_activation_key(activation_key, id)
}
