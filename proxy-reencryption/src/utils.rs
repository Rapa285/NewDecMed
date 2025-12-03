use std::{
    fmt::{self, Debug},
    str::FromStr,
    time::SystemTime,
};

use anyhow::{anyhow, Context};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use bip39::Mnemonic;
use chrono::{DateTime, Utc};
use iota_json_rpc_types::{
    DevInspectResults, IotaObjectData, IotaObjectDataFilter, IotaObjectDataOptions,
    IotaObjectResponseQuery, IotaTransactionBlockEffectsAPI,
};
use iota_keys::key_derive::derive_key_pair_from_path;
use iota_sdk::{IotaClient, IotaClientBuilder};
use iota_types::{
    base_types::{IotaAddress, ObjectID, ObjectRef},
    crypto::{EmptySignInfo, IotaKeyPair, Signature, SignatureScheme},
    message_envelope::Envelope,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{
        CallArg, ObjectArg, ProgrammableTransaction, SenderSignedData, TransactionData,
        TransactionDataAPI,
    },
    Identifier, TypeTag,
};
use jwt_simple::prelude::ES256KeyPair;
use move_core_types::{account_address::AccountAddress, language_storage::StructTag};
use rand::Rng;
use reqwest::{Client, IntoUrl};
use serde::{
    de::{self, DeserializeOwned},
    Deserialize, Deserializer, Serialize,
};
use serde_json::json;

use base64::{engine::general_purpose::STANDARD, Engine as _};

use crate::{
    constants::{GAS_STATION_BASE_URL, IOTA_URL, IPFS_BASE_URL, IPFS_GATEWAY_BASE_URL},
    current_fn,
    proxy_error::ProxyError,
    types::{ExecuteTxResponse, ReserveGasResponse, SuccessResponse, UtilIpfsAddResponse},
};

pub struct Utils {}

impl Utils {
    pub async fn add_and_pin_to_ipfs(data: String) -> Result<String, ProxyError> {
        let path_part = reqwest::multipart::Part::text(data);
        let form = reqwest::multipart::Form::new().part("path", path_part);
        let req_client = reqwest::Client::new();
        let res = req_client
            .post(format!("{}/add", IPFS_BASE_URL))
            .multipart(form)
            .send()
            .await
            .context(current_fn!())?;

        let res = res
            .json::<UtilIpfsAddResponse>()
            .await
            .context(current_fn!())?;

        Ok(res.cid)
    }

    pub fn build_success_response<T>(data: T, status_code: StatusCode) -> Response
    where
        T: Debug + Serialize,
    {
        (
            status_code,
            Json(SuccessResponse {
                data,
                status_code: status_code.as_u16(),
            }),
        )
            .into_response()
    }

    pub async fn construct_capability_call_arg(
        iota_client: &IotaClient,
        capability_id: ObjectID,
    ) -> Result<CallArg, ProxyError> {
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

    pub fn construct_es256_key_pair_from_pem(
        key_pair_pem: &str,
    ) -> Result<ES256KeyPair, ProxyError> {
        Ok(ES256KeyPair::from_pem(key_pair_pem).context(current_fn!())?)
    }

    pub fn construct_identifier_from_str(identifier: &str) -> Result<Identifier, ProxyError> {
        Ok(Identifier::from_str(identifier).context(current_fn!())?)
    }

    pub fn construct_pt(
        function_name: &str,
        package: ObjectID,
        module: Identifier,
        type_arguments: Vec<TypeTag>,
        call_args: Vec<CallArg>,
    ) -> Result<ProgrammableTransaction, ProxyError> {
        let mut builder = ProgrammableTransactionBuilder::new();
        let function =
            Utils::construct_identifier_from_str(function_name).context(current_fn!())?;

        builder
            .move_call(package, module, function, type_arguments, call_args)
            .context(current_fn!())?;

        Ok(builder.finish())
    }

    pub fn construct_shared_object_call_arg(id: ObjectID, version: u64, mutable: bool) -> CallArg {
        let activation_key_table_arg = ObjectArg::SharedObject {
            id,
            initial_shared_version: version.into(),
            mutable,
        };

        CallArg::Object(activation_key_table_arg)
    }

    pub fn construct_signature_from_str(signature: &str) -> Result<Signature, ProxyError> {
        Ok(Signature::from_str(signature)
            .map_err(|e| anyhow!(e.to_string()).context(current_fn!()))?)
    }

    pub fn construct_sponsored_tx_data(
        sender: IotaAddress,
        gas_payment: Vec<ObjectRef>,
        pt: ProgrammableTransaction,
        gas_budget: u64,
        gas_price: u64,
        sponsor_address: IotaAddress,
    ) -> TransactionData {
        let mut tx_data = TransactionData::new_programmable(
            sender,
            gas_payment.clone(),
            pt,
            gas_budget,
            gas_price,
        );

        tx_data.gas_data_mut().payment = gas_payment;
        tx_data.gas_data_mut().owner = sponsor_address;

        tx_data
    }

    pub fn debug_print<T>(func_name: &str, data: T)
    where
        T: Debug,
    {
        println!("{}: {:#?}", func_name, data);
    }

    pub fn decode_authorization_header(bearer_token: Option<&str>) -> Result<String, ProxyError> {
        if bearer_token.is_none() {
            return Err(ProxyError::Anyhow {
                source: anyhow!("Authorization header not found"),
                code: StatusCode::UNAUTHORIZED,
            });
        }

        let bearer_token: Vec<&str> = bearer_token.unwrap().split(" ").collect();

        if bearer_token.len() != 2 || bearer_token[0] != "Bearer" {
            return Err(ProxyError::Anyhow {
                source: anyhow!("Failed to decode bearer token"),
                code: StatusCode::UNAUTHORIZED,
            });
        }

        Ok(bearer_token[1].to_string())
    }

    pub async fn do_http_get_request<T, E, U>(
        req_client: &Client,
        success_status_code: StatusCode,
        url: U,
    ) -> Result<T, ProxyError>
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

            return Err(ProxyError::Anyhow {
                source: anyhow!(format!("{:#?}", error)).context(current_fn!()),
                code: StatusCode::INTERNAL_SERVER_ERROR,
            });
        }

        match content_type.as_str() {
            "application/json" => {
                Ok(serde_json::from_slice(&res_body.to_vec()).context(current_fn!())?)
            }
            "text/plain; charset=utf-8" => Ok(T::from(
                String::from_utf8(res_body.to_vec()).context(current_fn!())?,
            )),
            _ => {
                return Err(ProxyError::Anyhow {
                    source: anyhow!(format!("Unknown content-type: {}", content_type))
                        .context(current_fn!()),
                    code: StatusCode::INTERNAL_SERVER_ERROR,
                })
            }
        }
    }

    pub fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
        T::Err: fmt::Display,
    {
        let opt = Option::<String>::deserialize(de)?;
        match opt.as_deref() {
            None | Some("") => Ok(None),
            Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
        }
    }

    pub async fn execute_tx(
        tx: Envelope<SenderSignedData, EmptySignInfo>,
        reservation_id: u64,
    ) -> Result<ExecuteTxResponse, ProxyError> {
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

        let ex_tx_res = res
            .json::<ExecuteTxResponse>()
            .await
            .context(current_fn!())?;

        Ok(ex_tx_res)
    }

    pub fn generate_64_bytes_seed() -> [u8; 64] {
        let mut rng = rand::rng();
        let mut random_seed = [0u8; 64];
        rng.fill(&mut random_seed);

        random_seed
    }

    pub fn generate_iota_keys_ed(seed: &[u8]) -> Result<(IotaAddress, IotaKeyPair), ProxyError> {
        Ok(derive_key_pair_from_path(
            &seed,
            Some(bip32::DerivationPath::from_str("m/44'/4218'/0'/0'/0'").unwrap()),
            &SignatureScheme::ED25519,
        )
        .context(current_fn!())?)
    }

    /**
     * Return: `(public_key, secret_key)`
     */
    pub fn generate_jwt() -> Result<(String, String), ProxyError> {
        let keypair = ES256KeyPair::generate();

        let secret_key = keypair.to_pem().context(current_fn!())?;
        let public_key = keypair.public_key().to_pem().context(current_fn!())?;

        Ok((public_key, secret_key))
    }

    pub fn generate_mnemonic(size: usize) -> Result<Mnemonic, ProxyError> {
        Ok(Mnemonic::generate(size).context(current_fn!())?)
    }

    pub async fn get_data_ipfs(cid: String) -> Result<String, ProxyError> {
        let req_client = reqwest::Client::new();
        let content = Utils::do_http_get_request::<String, String, _>(
            &req_client,
            StatusCode::OK,
            format!("{}/ipfs/{}", IPFS_GATEWAY_BASE_URL, cid),
        )
        .await
        .context(current_fn!())?;

        Ok(content)
    }

    pub async fn get_iota_client() -> Result<IotaClient, ProxyError> {
        Ok(IotaClientBuilder::default()
            .build(IOTA_URL)
            .await
            .context(current_fn!())?)
    }

    pub async fn get_proxy_cap(
        iota_client: &IotaClient,
        module: Identifier,
        package_id: AccountAddress,
        proxy_address: IotaAddress,
    ) -> Result<IotaObjectData, ProxyError> {
        let query = IotaObjectResponseQuery {
            filter: Some(IotaObjectDataFilter::StructType(StructTag {
                address: package_id,
                module,
                name: Identifier::from_str("ProxyCap").context(current_fn!())?,
                type_params: vec![],
            })),
            options: None,
        };
        let res = iota_client
            .read_api()
            .get_owned_objects(proxy_address, query, None, 1)
            .await
            .context(current_fn!())?;

        if res.data.is_empty() {
            return Err(ProxyError::Anyhow {
                source: anyhow!("ProxyCap not found").context(current_fn!()),
                code: StatusCode::INTERNAL_SERVER_ERROR,
            });
        }

        let proxy_cap = res.data[0].data.clone().unwrap();

        Ok(proxy_cap)
    }

    pub async fn get_ref_gas_price(iota_client: &IotaClient) -> Result<u64, ProxyError> {
        Ok((*iota_client)
            .governance_api()
            .get_reference_gas_price()
            .await
            .context(current_fn!())?)
    }

    pub fn handle_error_execute_tx(response: ExecuteTxResponse) -> Result<(), ProxyError> {
        if response.error.is_some() {
            return Err(ProxyError::Anyhow {
                source: anyhow!(response.error.unwrap()).context(current_fn!()),
                code: StatusCode::INTERNAL_SERVER_ERROR,
            });
        }

        if response.effects.as_ref().unwrap().status().is_err() {
            return Err(ProxyError::Anyhow {
                source: anyhow!(response.effects.unwrap().status().to_string())
                    .context(current_fn!()),
                code: StatusCode::INTERNAL_SERVER_ERROR,
            });
        }

        Ok(())
    }

    pub fn handle_error_move_call_read_only(response: DevInspectResults) -> Result<(), ProxyError> {
        if response.error.is_some() {
            return Err(ProxyError::Anyhow {
                source: anyhow!(response.error.unwrap()).context(current_fn!()),
                code: StatusCode::INTERNAL_SERVER_ERROR,
            });
        }

        if response.effects.status().is_err() {
            return Err(ProxyError::Anyhow {
                source: anyhow!(response.effects.status().to_string()).context(current_fn!()),
                code: StatusCode::INTERNAL_SERVER_ERROR,
            });
        }

        Ok(())
    }

    pub async fn move_call_read_only(
        sender: IotaAddress,
        iota_client: &IotaClient,
        pt: ProgrammableTransaction,
    ) -> Result<DevInspectResults, ProxyError> {
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

    pub fn parse_move_read_only_result<T: DeserializeOwned>(
        val: DevInspectResults,
        index: usize,
    ) -> Result<T, ProxyError> {
        let res = val.results.unwrap()[0].return_values[index].0.to_vec();

        Ok(bcs::from_bytes::<T>(res.as_slice()).context(current_fn!())?)
    }

    pub async fn reserve_gas(
        gas_budget: u64,
        reserve_duration_secs: u64,
    ) -> Result<(IotaAddress, u64, Vec<ObjectRef>), ProxyError> {
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
            .context(current_fn!())?)
    }

    pub fn serde_deserialize_from_base64<T>(val: String) -> Result<T, ProxyError>
    where
        T: DeserializeOwned,
    {
        let val = STANDARD.decode(val).context(current_fn!())?;
        let ori_val: T = serde_json::from_slice(&val).context(current_fn!())?;

        Ok(ori_val)
    }

    pub fn serde_serialize_to_base64<T>(val: &T) -> Result<String, ProxyError>
    where
        T: Serialize,
    {
        let ser_val = serde_json::to_vec(val).context(current_fn!())?;
        Ok(STANDARD.encode(ser_val))
    }

    pub fn sys_time_to_iso(system_time: SystemTime) -> String {
        let iso: DateTime<Utc> = system_time.into();
        iso.to_rfc3339()
    }
}
