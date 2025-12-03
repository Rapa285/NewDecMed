use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use anyhow::{anyhow, Context};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Algorithm, Argon2, Params, PasswordHash, PasswordVerifier, Version,
};
use iota_json_rpc_types::{
    DevInspectResults, IotaObjectDataOptions, IotaTransactionBlockEffectsAPI,
};
use iota_sdk::{IotaClient, IotaClientBuilder};
use iota_types::base_types::{IotaAddress, ObjectID, ObjectRef};
use iota_types::crypto::{EmptySignInfo, IotaKeyPair};
use iota_types::message_envelope::Envelope;
use iota_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use iota_types::transaction::{
    CallArg, ObjectArg, ProgrammableTransaction, SenderSignedData, TransactionData,
    TransactionDataAPI,
};
use iota_types::{Identifier, TypeTag};
use rand::Rng;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use tauri::http::StatusCode;
use tauri_plugin_http::reqwest::{self, Client, IntoUrl};
use umbral_pre::{PublicKey, SecretKey, SecretKeyFactory};

use crate::{
    client_error::ClientError,
    constants::{GAS_STATION_BASE_URL, HASH_SALT},
};
use crate::{
    constants::IOTA_URL,
    current_fn,
    types::{ExecuteTxResponse, KeysEntry, ReserveGasResponse},
};
use base64::{engine::general_purpose::STANDARD, Engine as _};

pub async fn reserve_gas(
    gas_budget: u64,
    reserve_duration_secs: u64,
) -> Result<(IotaAddress, u64, Vec<ObjectRef>), ClientError> {
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
) -> Result<ExecuteTxResponse, ClientError> {
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

pub fn parse_keys_entry(keys_entry: &Vec<u8>) -> Result<KeysEntry, ClientError> {
    Ok(serde_json::from_slice(keys_entry).context(current_fn!())?)
}

pub fn generate_64_bytes_seed() -> [u8; 64] {
    let mut rng = rand::rng();
    let mut random_seed = [0u8; 64];
    rng.fill(&mut random_seed);

    random_seed
}

pub fn construct_pt(
    function_name: String,
    package: ObjectID,
    module: Identifier,
    type_arguments: Vec<TypeTag>,
    call_args: Vec<CallArg>,
) -> Result<ProgrammableTransaction, ClientError> {
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

pub async fn get_ref_gas_price(iota_client: &IotaClient) -> Result<u64, ClientError> {
    Ok((*iota_client)
        .governance_api()
        .get_reference_gas_price()
        .await
        .context(current_fn!())?)
}

pub async fn construct_capability_call_arg(
    iota_client: &IotaClient,
    capability_id: ObjectID,
) -> Result<CallArg, ClientError> {
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
) -> Result<DevInspectResults, ClientError> {
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

pub fn argon_hash(password: String) -> Result<String, ClientError> {
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

pub fn _argon_verify(hash: String, password: String) -> Result<bool, ClientError> {
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

pub fn _sha_hash_to_hex(data: &[u8]) -> String {
    let hash = Sha256::digest(data);
    hex::encode(hash)
}

pub fn compute_pre_keys(seed: &[u8]) -> Result<(SecretKey, PublicKey), ClientError> {
    let secret_key = SecretKeyFactory::from_secure_randomness(seed)
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?
        .make_key(seed);
    let public_key = secret_key.public_key();

    Ok((secret_key, public_key))
}

pub fn parse_move_read_only_result<T: DeserializeOwned>(
    val: DevInspectResults,
    index: usize,
) -> Result<T, ClientError> {
    let res = val.results.context(current_fn!())?[0].return_values[index]
        .0
        .to_vec();

    Ok(bcs::from_bytes::<T>(&res).context(current_fn!())?)
}

pub async fn get_iota_client() -> Result<IotaClient, ClientError> {
    Ok(IotaClientBuilder::default()
        .build(IOTA_URL)
        .await
        .context(current_fn!())?)
}

pub fn handle_error_move_call_read_only(response: DevInspectResults) -> Result<(), ClientError> {
    if response.error.is_some() {
        return Err(ClientError::Anyhow(
            anyhow!(response.error.unwrap()).context(current_fn!()),
        ));
    }

    if response.effects.status().is_err() {
        return Err(ClientError::Anyhow(
            anyhow!(response.effects.status().to_string()).context(current_fn!()),
        ));
    }

    Ok(())
}

/**
 * Return: `(id_part, hospital_part)`
 */
pub fn decode_hospital_personnel_id(id: String) -> Result<(String, String), ClientError> {
    let id: Vec<&str> = id.split("@").collect();

    if id.len() != 2 {
        return Err(ClientError::Anyhow(
            anyhow!("Invalid id").context(current_fn!()),
        ));
    }

    Ok((id[0].to_string(), id[1].to_string()))
}

pub fn handle_error_execute_tx(response: ExecuteTxResponse) -> Result<u64, ClientError> {
    if response.error.is_some() {
        return Err(ClientError::Anyhow(
            anyhow!(response.error.unwrap()).context(current_fn!()),
        ));
    }

    if response.effects.is_some() && response.effects.as_ref().unwrap().status().is_err() {
        return Err(ClientError::Anyhow(
            anyhow!(response.effects.unwrap().status().to_string()).context(current_fn!()),
        ));
    }

    Ok(0)
}

pub fn _decode_hospital_personnel_qr(
    content: String,
) -> Result<(IotaAddress, PublicKey), ClientError> {
    let content: Vec<&str> = content.split("@").collect();

    if content.len() != 2 {
        return Err(ClientError::Anyhow(
            anyhow!("Invalid content length, expected 2 found {}", content.len())
                .context(current_fn!()),
        ));
    }

    let iota_address = IotaAddress::from_str(content[0]).context(current_fn!())?;
    let pre_public_key =
        serde_deserialize_from_base64(content[1].to_string()).context(current_fn!())?;

    Ok((iota_address, pre_public_key))
}

pub fn get_global_admin_iota_address_from_keys_entry(
    keys_entry: &KeysEntry,
) -> Result<IotaAddress, ClientError> {
    Ok(
        IotaAddress::from_str(&keys_entry.admin_address.as_ref().ok_or(
            anyhow!("Global admin iota address not found on keys entry").context(current_fn!()),
        )?)
        .context(current_fn!())?,
    )
}

pub fn get_global_admin_iota_key_pair_from_keys_entry(
    keys_entry: &KeysEntry,
) -> Result<IotaKeyPair, ClientError> {
    Ok(
        IotaKeyPair::decode(&keys_entry.admin_secret_key.as_ref().ok_or(
            anyhow!("Global admin iota key pair not found on keys entry").context(current_fn!()),
        )?)
        .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?,
    )
}

pub fn get_global_admin_pre_keys_from_keys_entry(
    keys_entry: &KeysEntry,
) -> Result<(SecretKey, PublicKey), ClientError> {
    let pre_seed = STANDARD
        .decode(keys_entry.admin_pre_seed.clone())
        .context(current_fn!())?;
    let (pre_secret_key, pre_public_key) = compute_pre_keys(&pre_seed).context(current_fn!())?;

    Ok((pre_secret_key, pre_public_key))
}

pub fn serde_serialize_to_base64<T>(val: &T) -> Result<String, ClientError>
where
    T: Serialize,
{
    let ser_val = serde_json::to_vec(val).context(current_fn!())?;
    Ok(STANDARD.encode(ser_val))
}

pub fn serde_deserialize_from_base64<T>(val: String) -> Result<T, ClientError>
where
    T: DeserializeOwned,
{
    let val = STANDARD.decode(val).context(current_fn!())?;
    let ori_val: T = serde_json::from_slice(&val).context(current_fn!())?;

    Ok(ori_val)
}

pub async fn _do_http_get_request_text<T, E, U>(
    access_token: Option<String>,
    req_client: &Client,
    success_status_code: StatusCode,
    url: U,
) -> Result<T, ClientError>
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

        return Err(ClientError::Anyhow(
            anyhow!(format!("{:#?}", error)).context(current_fn!()),
        ));
    }

    match content_type.as_str() {
        "text/plain; charset=utf-8" => Ok(T::from_str(
            &String::from_utf8(res_body.to_vec()).context(current_fn!())?,
        )
        .map_err(|e| anyhow!(e.to_string()))?),
        _ => {
            return Err(ClientError::Anyhow(
                anyhow!(format!("Unknown content-type: {}", content_type)).context(current_fn!()),
            ))
        }
    }
}
