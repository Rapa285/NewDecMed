use anyhow::{anyhow, Context};
use std::str::FromStr; // ← diperlukan untuk Identifier::from_str

use crate::{
    constants::{GAS_STATION_BASE_URL, IOTA_URL},
    current_fn,
    audit_error::AuditError,
    types::{ExecuteTxResponse, ReserveGasResponse},
};

use iota_json_rpc_types::{
    DevInspectResults, IotaTransactionBlockEffectsAPI,
};
use iota_sdk::{IotaClient, IotaClientBuilder};
use iota_types::{
    base_types::{IotaAddress, ObjectID, ObjectRef},
    crypto::EmptySignInfo,
    message_envelope::Envelope,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{
        CallArg, ProgrammableTransaction, SenderSignedData, TransactionData,
        TransactionDataAPI, Transaction,
    },
    Identifier, TypeTag,
};
use serde_json::json;
use serde::de::DeserializeOwned;
use axum::http::StatusCode;

pub struct IotaUtils {}

impl IotaUtils {
    pub async fn reserve_gas(
        gas_budget: u64,
        reserve_duration_secs: u64,
    ) -> Result<(IotaAddress, u64, Vec<ObjectRef>), AuditError> {
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

    pub async fn get_ref_gas_price(iota_client: &IotaClient) -> Result<u64, AuditError> {
        Ok((*iota_client)
            .governance_api()
            .get_reference_gas_price()
            .await
            .context(current_fn!())?)
    }

    pub async fn execute_tx(
        tx: Transaction, // ← diganti dari Envelope<SenderSignedData, EmptySignInfo>
        reservation_id: u64,
    ) -> Result<ExecuteTxResponse, AuditError> {
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

    pub fn construct_pt(
        function_name: String,
        package: ObjectID,
        module: Identifier,
        type_arguments: Vec<TypeTag>,
        call_args: Vec<CallArg>,
    ) -> Result<ProgrammableTransaction, AuditError> {
        let mut builder = ProgrammableTransactionBuilder::new();
        // Gunakan Identifier::new() bukan from_str (dari move-core-types)
        let function = Identifier::new(function_name.as_str())
            .map_err(|e| anyhow::anyhow!("nama fungsi tidak valid: {e}"))
            .context(current_fn!())?;

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

    pub async fn move_call_read_only(
        sender: IotaAddress,
        iota_client: &IotaClient,
        pt: ProgrammableTransaction,
    ) -> Result<DevInspectResults, AuditError> {
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
    ) -> Result<T, AuditError> {
        let res = val.results.context(current_fn!())?[0].return_values[index]
            .0
            .to_vec();

        Ok(bcs::from_bytes::<T>(&res).context(current_fn!())?)
    }

    pub async fn get_iota_client() -> Result<IotaClient, AuditError> {
        Ok(IotaClientBuilder::default()
            .build(IOTA_URL)
            .await
            .context(current_fn!())?)
    }

    pub fn handle_error_move_call_read_only(response: DevInspectResults) -> Result<(), AuditError> {
        if response.error.is_some() {
            return Err(AuditError::Anyhow {
                source: anyhow!(response.error.unwrap()).context(current_fn!()),
                code: StatusCode::INTERNAL_SERVER_ERROR,
            });
        }

        if response.effects.status().is_err() {
            return Err(AuditError::Anyhow {
                source: anyhow!(response.effects.status().to_string()).context(current_fn!()),
                code: StatusCode::INTERNAL_SERVER_ERROR,
            });
        }

        Ok(())
    }

    pub fn handle_error_execute_tx(response: ExecuteTxResponse) -> Result<u64, AuditError> {
        if response.error.is_some() {
            return Err(AuditError::Anyhow {
                source: anyhow!(response.error.unwrap()).context(current_fn!()),
                code: StatusCode::INTERNAL_SERVER_ERROR,
            });
        }

        if response.effects.is_some() && response.effects.as_ref().unwrap().status().is_err() {
            return Err(AuditError::Anyhow {
                source: anyhow!(response.effects.unwrap().status().to_string()).context(current_fn!()),
                code: StatusCode::INTERNAL_SERVER_ERROR,
            });
        }

        Ok(0)
    }
}