use anyhow::Context;
use crate::{
    // client_error::ClientError,
    constants::GAS_BUDGET,
    current_fn,
    // types::{DecmedPackage, MoveHospital},
    // utils::{
    //     construct_capability_call_arg, construct_pt, construct_shared_object_call_arg,
    //     construct_sponsored_tx_data, execute_tx, get_iota_client, get_ref_gas_price,
    //     handle_error_execute_tx, handle_error_move_call_read_only, move_call_read_only,
    //     parse_move_read_only_result, reserve_gas,
    // },
};

pub struct Iota_utils{}

pub impl Iota_utils{
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

    pub async fn get_ref_gas_price(iota_client: &IotaClient) -> Result<u64, ProxyError> {
        Ok((*iota_client)
            .governance_api()
            .get_reference_gas_price()
            .await
            .context(current_fn!())?)
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

    pub async fn get_all_logs(&self) -> Result<Vec<IotaLogMetadata>, AuditError> {
        let client = get_iota_client()
            .await
            .context(current_fn!())?;

        // Filter berdasarkan tipe object LogRecord
        let struct_type_str = format!("{}::audit_log::LogRecord", self.package_id);

        let struct_tag = StructTag::from_str(&struct_type_str)
            .context(current_fn!())?;

        let query = IotaObjectResponseQuery::new()
            .with_filter(IotaObjectDataFilter::StructType(struct_tag))
            .with_options(IotaObjectDataOptions::new().with_content());

        let mut all_logs: Vec<IotaLogMetadata> = Vec::new();
        let mut cursor = None; 

        // Loop pagination (karena IOTA membatasi jumlah data per query)
        loop {
            let page = client
                .read_api()
                .query_objects(query.clone(), cursor, Some(50))
                .await
                .context(current_fn!())?;

            for obj in page.data {
                if let Some(content) = obj.data.and_then(|d| d.content) {
                    if let iota_sdk::rpc_types::IotaParsedData::MoveObject(move_obj) = content {
                        if let Ok(fields_json) = serde_json::to_value(move_obj.fields) {
                            if let Some(json_str) = fields_json.get("json_data").and_then(|v| v.as_str()) {
                                if let Ok(metadata) = serde_json::from_str::<IotaLogMetadata>(json_str) {
                                    all_logs.push(metadata);
                                }
                            }
                        }
                    }
                }
            }

            if page.has_next_page {
                cursor = page.next_cursor;
            } else {
                break;
            }
        }

        // Opsional: Urutkan dari log pertama (terlama) ke terbaru berdasarkan log_sequence_number
        all_logs.sort_by_key(|log| log.log_sequence_number);
        
        Ok(all_logs)
    }
}
