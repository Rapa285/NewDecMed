use std::str::FromStr;

use anyhow::Context;
use iota_types::{
    base_types::{IotaAddress, ObjectID},
    crypto::IotaKeyPair,
    gas_coin::NANOS_PER_IOTA,
    transaction::{CallArg, Transaction},
};

use crate::{
    constants::GAS_BUDGET,
    current_fn,
    hospital_error::HospitalError,
    types::{
        DecmedPackage, HospitalPersonnelRole, MoveHospitalMetadata,
        MoveHospitalPersonnelAccessData, MoveHospitalPersonnelAdministrativeMetadata,
        MoveHospitalPersonnelMetadata,
    },
    utils::{
        construct_capability_call_arg, construct_pt, construct_shared_object_call_arg,
        construct_sponsored_tx_data, execute_tx, get_iota_client, get_ref_gas_price,
        handle_error_execute_tx, handle_error_move_call_read_only, move_call_read_only,
        parse_move_read_only_result, reserve_gas,
    },
};

pub struct MoveCall {
    pub decmed_package: DecmedPackage,
}

impl MoveCall {
    pub fn construct_address_id_object_call_arg(&self, mutable: bool) -> CallArg {
        construct_shared_object_call_arg(
            self.decmed_package.address_id_object_id,
            self.decmed_package.address_id_object_version,
            mutable,
        )
    }

    pub fn construct_clock_call_arg(&self) -> CallArg {
        construct_shared_object_call_arg(ObjectID::from_str("0x6").unwrap(), 1, false)
    }

    pub async fn construct_global_admin_cap(&self) -> Result<CallArg, HospitalError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        Ok(
            construct_capability_call_arg(&iota_client, self.decmed_package.global_admin_cap_id)
                .await
                .context(current_fn!())?,
        )
    }

    pub fn construct_hospital_id_metadata_object_call_arg(&self, mutable: bool) -> CallArg {
        construct_shared_object_call_arg(
            self.decmed_package.hospital_id_metadata_object_id,
            self.decmed_package.hospital_id_metadata_object_version,
            mutable,
        )
    }

    pub fn construct_hospital_personnel_id_account_object_call_arg(
        &self,
        mutable: bool,
    ) -> CallArg {
        construct_shared_object_call_arg(
            self.decmed_package.hospital_personnel_id_account_object_id,
            self.decmed_package
                .hospital_personnel_id_account_object_version,
            mutable,
        )
    }

    pub async fn cleanup_read_access(
        &self,
        activation_key: String,
        sender: IotaAddress,
        sender_key_pair: IotaKeyPair,
    ) -> Result<(), HospitalError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            "cleanup_read_access".to_string(),
            self.decmed_package.package_id,
            self.decmed_package.module_hospital_personnel.clone(),
            vec![],
            vec![
                CallArg::Pure(bcs::to_bytes(&activation_key).context(current_fn!())?),
                self.construct_address_id_object_call_arg(false),
                self.construct_clock_call_arg(),
                self.construct_hospital_personnel_id_account_object_call_arg(true),
            ],
        )
        .context(current_fn!())?;

        let (sponsor_account, reservation_id, gas_coins) = reserve_gas(NANOS_PER_IOTA * 2, 10)
            .await
            .context(current_fn!())?;
        let ref_gas_price = get_ref_gas_price(&iota_client)
            .await
            .context(current_fn!())?;

        let tx_data = construct_sponsored_tx_data(
            sender,
            gas_coins,
            pt,
            GAS_BUDGET,
            ref_gas_price,
            sponsor_account,
        );

        let signer = sender_key_pair;
        let tx = Transaction::from_data_and_signer(tx_data, vec![&signer]);

        let response = execute_tx(tx, reservation_id)
            .await
            .context(current_fn!())?;

        handle_error_execute_tx(response).context(current_fn!())?;

        Ok(())
    }

    pub async fn cleanup_update_access(
        &self,
        activation_key: String,
        sender: IotaAddress,
        sender_key_pair: IotaKeyPair,
    ) -> Result<(), HospitalError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            "cleanup_update_access".to_string(),
            self.decmed_package.package_id,
            self.decmed_package.module_hospital_personnel.clone(),
            vec![],
            vec![
                CallArg::Pure(bcs::to_bytes(&activation_key).context(current_fn!())?),
                self.construct_address_id_object_call_arg(false),
                self.construct_clock_call_arg(),
                self.construct_hospital_personnel_id_account_object_call_arg(true),
            ],
        )
        .context(current_fn!())?;

        let (sponsor_account, reservation_id, gas_coins) = reserve_gas(NANOS_PER_IOTA * 2, 10)
            .await
            .context(current_fn!())?;
        let ref_gas_price = get_ref_gas_price(&iota_client)
            .await
            .context(current_fn!())?;

        let tx_data = construct_sponsored_tx_data(
            sender,
            gas_coins,
            pt,
            GAS_BUDGET,
            ref_gas_price,
            sponsor_account,
        );

        let signer = sender_key_pair;
        let tx = Transaction::from_data_and_signer(tx_data, vec![&signer]);

        let response = execute_tx(tx, reservation_id)
            .await
            .context(current_fn!())?;

        handle_error_execute_tx(response).context(current_fn!())?;

        Ok(())
    }

    pub async fn get_account_info(
        &self,
        activation_key: String,
        sender: IotaAddress,
    ) -> Result<
        (
            Option<MoveHospitalPersonnelAdministrativeMetadata>,
            HospitalPersonnelRole,
            MoveHospitalMetadata,
        ),
        HospitalError,
    > {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            String::from("get_account_info"),
            self.decmed_package.package_id,
            self.decmed_package.module_hospital_personnel.clone(),
            vec![],
            vec![
                CallArg::Pure(bcs::to_bytes(&activation_key).context(current_fn!())?),
                self.construct_address_id_object_call_arg(false),
                self.construct_hospital_id_metadata_object_call_arg(false),
                self.construct_hospital_personnel_id_account_object_call_arg(false),
            ],
        )
        .context(current_fn!())?;

        let response = move_call_read_only(sender, &iota_client, pt)
            .await
            .context(current_fn!())?;
        handle_error_move_call_read_only(response.clone()).context(current_fn!())?;

        let hospital_personnel_administrative_metadata: Option<
            MoveHospitalPersonnelAdministrativeMetadata,
        > = parse_move_read_only_result(response.clone(), 0).context(current_fn!())?;
        let role: HospitalPersonnelRole =
            parse_move_read_only_result(response.clone(), 1).context(current_fn!())?;
        let hospital_metadata: MoveHospitalMetadata =
            parse_move_read_only_result(response.clone(), 2).context(current_fn!())?;

        Ok((
            hospital_personnel_administrative_metadata,
            role,
            hospital_metadata,
        ))
    }

    /// ## Return:
    /// 0: status_code
    ///     - 0 means need activation
    ///     - 1 means need signup
    ///     - 2 means need signin
    ///     - 3 means ok
    pub async fn get_account_state(
        &self,
        activation_key: String,
        hospital_id: String,
        personnel_id: String,
        sender: IotaAddress,
    ) -> Result<(u64, Option<HospitalPersonnelRole>), HospitalError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            String::from("get_account_state"),
            self.decmed_package.package_id,
            self.decmed_package.module_hospital_personnel.clone(),
            vec![],
            vec![
                CallArg::Pure(bcs::to_bytes(&activation_key).context(current_fn!())?),
                CallArg::Pure(bcs::to_bytes(&hospital_id).context(current_fn!())?),
                self.construct_hospital_personnel_id_account_object_call_arg(false),
                CallArg::Pure(bcs::to_bytes(&personnel_id).context(current_fn!())?),
            ],
        )
        .context(current_fn!())?;

        let response = move_call_read_only(sender, &iota_client, pt)
            .await
            .context(current_fn!())?;
        handle_error_move_call_read_only(response.clone()).context(current_fn!())?;

        let state: u64 = parse_move_read_only_result(response.clone(), 0).context(current_fn!())?;
        let role: Option<HospitalPersonnelRole> =
            parse_move_read_only_result(response, 1).context(current_fn!())?;

        Ok((state, role))
    }

    pub async fn get_hospital_personnels(
        &self,
        activation_key: String,
        sender: IotaAddress,
    ) -> Result<Vec<MoveHospitalPersonnelMetadata>, HospitalError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            String::from("get_hospital_personnels"),
            self.decmed_package.package_id,
            self.decmed_package.module_hospital_personnel.clone(),
            vec![],
            vec![
                CallArg::Pure(bcs::to_bytes(&activation_key).context(current_fn!())?),
                self.construct_address_id_object_call_arg(false),
                self.construct_hospital_personnel_id_account_object_call_arg(false),
            ],
        )
        .context(current_fn!())?;

        let response = move_call_read_only(sender, &iota_client, pt)
            .await
            .context(current_fn!())?;
        handle_error_move_call_read_only(response.clone()).context(current_fn!())?;

        let hospital_peersonnels_metadata: Vec<MoveHospitalPersonnelMetadata> =
            parse_move_read_only_result(response.clone(), 0).context(current_fn!())?;

        Ok(hospital_peersonnels_metadata)
    }

    pub async fn get_read_access(
        &self,
        activation_key: String,
        sender: IotaAddress,
    ) -> Result<Vec<MoveHospitalPersonnelAccessData>, HospitalError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            String::from("get_read_access"),
            self.decmed_package.package_id,
            self.decmed_package.module_hospital_personnel.clone(),
            vec![],
            vec![
                CallArg::Pure(bcs::to_bytes(&activation_key).context(current_fn!())?),
                self.construct_address_id_object_call_arg(false),
                self.construct_hospital_personnel_id_account_object_call_arg(false),
            ],
        )
        .context(current_fn!())?;

        let response = move_call_read_only(sender, &iota_client, pt)
            .await
            .context(current_fn!())?;
        handle_error_move_call_read_only(response.clone()).context(current_fn!())?;

        let access: Vec<MoveHospitalPersonnelAccessData> =
            parse_move_read_only_result(response.clone(), 0).context(current_fn!())?;

        Ok(access)
    }

    pub async fn get_update_access(
        &self,
        activation_key: String,
        sender: IotaAddress,
    ) -> Result<Vec<MoveHospitalPersonnelAccessData>, HospitalError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            String::from("get_update_access"),
            self.decmed_package.package_id,
            self.decmed_package.module_hospital_personnel.clone(),
            vec![],
            vec![
                CallArg::Pure(bcs::to_bytes(&activation_key).context(current_fn!())?),
                self.construct_address_id_object_call_arg(false),
                self.construct_hospital_personnel_id_account_object_call_arg(false),
            ],
        )
        .context(current_fn!())?;

        let response = move_call_read_only(sender, &iota_client, pt)
            .await
            .context(current_fn!())?;
        handle_error_move_call_read_only(response.clone()).context(current_fn!())?;

        let access: Vec<MoveHospitalPersonnelAccessData> =
            parse_move_read_only_result(response.clone(), 0).context(current_fn!())?;

        Ok(access)
    }

    pub async fn global_admin_create_activation_key(
        &self,
        activation_key: String,
        hospital_admin_id: String,
        hospital_id: String,
        hospital_name: String,
        sender: IotaAddress,
        sender_key_pair: IotaKeyPair,
    ) -> Result<(), HospitalError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            String::from("create_activation_key"),
            self.decmed_package.package_id,
            self.decmed_package.module_admin.clone(),
            vec![],
            vec![
                CallArg::Pure(bcs::to_bytes(&activation_key).context(current_fn!())?),
                CallArg::Pure(bcs::to_bytes(&hospital_admin_id).context(current_fn!())?),
                CallArg::Pure(bcs::to_bytes(&hospital_id).context(current_fn!())?),
                self.construct_hospital_id_metadata_object_call_arg(true),
                CallArg::Pure(bcs::to_bytes(&hospital_name).context(current_fn!())?),
                self.construct_hospital_personnel_id_account_object_call_arg(true),
                self.construct_global_admin_cap()
                    .await
                    .context(current_fn!())?,
            ],
        )
        .context(current_fn!())?;

        let (sponsor_account, reservation_id, gas_coins) = reserve_gas(NANOS_PER_IOTA, 10)
            .await
            .context(current_fn!())?;
        let ref_gas_price = get_ref_gas_price(&iota_client)
            .await
            .context(current_fn!())?;

        let tx_data = construct_sponsored_tx_data(
            sender,
            gas_coins,
            pt,
            GAS_BUDGET,
            ref_gas_price,
            sponsor_account,
        );

        let signer = sender_key_pair;
        let tx = Transaction::from_data_and_signer(tx_data, vec![&signer]);

        let response = execute_tx(tx, reservation_id)
            .await
            .context(current_fn!())?;

        handle_error_execute_tx(response).context(current_fn!())?;

        Ok(())
    }

    pub async fn is_account_registered(
        &self,
        activation_key: String,
        sender: IotaAddress,
    ) -> Result<bool, HospitalError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            String::from("is_account_registered"),
            self.decmed_package.package_id,
            self.decmed_package.module_hospital_personnel.clone(),
            vec![],
            vec![
                CallArg::Pure(bcs::to_bytes(&activation_key).context(current_fn!())?),
                self.construct_address_id_object_call_arg(false),
                self.construct_hospital_personnel_id_account_object_call_arg(false),
            ],
        )
        .context(current_fn!())?;

        let response = move_call_read_only(sender, &iota_client, pt)
            .await
            .context(current_fn!())?;
        handle_error_move_call_read_only(response.clone()).context(current_fn!())?;

        Ok(true)
    }

    pub async fn hospital_admin_create_activation_key(
        &self,
        admin_activation_key: String,
        metadata: String,
        personnel_activation_key: String,
        personnel_id: String,
        role: &str,
        sender: IotaAddress,
        sender_key_pair: IotaKeyPair,
    ) -> Result<(), HospitalError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            String::from("create_activation_key"),
            self.decmed_package.package_id,
            self.decmed_package.module_hospital_personnel.clone(),
            vec![],
            vec![
                self.construct_address_id_object_call_arg(false),
                CallArg::Pure(bcs::to_bytes(&admin_activation_key).context(current_fn!())?),
                self.construct_hospital_personnel_id_account_object_call_arg(true),
                CallArg::Pure(bcs::to_bytes(&metadata).context(current_fn!())?),
                CallArg::Pure(bcs::to_bytes(&personnel_activation_key).context(current_fn!())?),
                CallArg::Pure(bcs::to_bytes(&personnel_id).context(current_fn!())?),
                CallArg::Pure(bcs::to_bytes(role.as_bytes()).context(current_fn!())?),
            ],
        )
        .context(current_fn!())?;
        let (sponsor_account, reservation_id, gas_coins) = reserve_gas(NANOS_PER_IOTA * 2, 10)
            .await
            .context(current_fn!())?;
        let ref_gas_price = get_ref_gas_price(&iota_client)
            .await
            .context(current_fn!())?;

        let tx_data = construct_sponsored_tx_data(
            sender,
            gas_coins,
            pt,
            GAS_BUDGET,
            ref_gas_price,
            sponsor_account,
        );

        let signer = sender_key_pair;
        let tx = Transaction::from_data_and_signer(tx_data, vec![&signer]);

        let response = execute_tx(tx, reservation_id)
            .await
            .context(current_fn!())?;

        handle_error_execute_tx(response).context(current_fn!())?;

        Ok(())
    }

    pub async fn signup(
        &self,
        activation_key: String,
        hospital_id: String,
        personnel_id: String,
        private_metadata: String,
        public_metadata: String,
        sender: IotaAddress,
        sender_key_pair: IotaKeyPair,
    ) -> Result<(), HospitalError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            String::from("signup"),
            self.decmed_package.package_id,
            self.decmed_package.module_hospital_personnel.clone(),
            vec![],
            vec![
                CallArg::Pure(bcs::to_bytes(&activation_key).context(current_fn!())?),
                self.construct_address_id_object_call_arg(true),
                CallArg::Pure(bcs::to_bytes(&hospital_id).context(current_fn!())?),
                self.construct_hospital_personnel_id_account_object_call_arg(true),
                CallArg::Pure(bcs::to_bytes(&personnel_id).context(current_fn!())?),
                CallArg::Pure(bcs::to_bytes(&private_metadata).context(current_fn!())?),
                CallArg::Pure(bcs::to_bytes(&public_metadata).context(current_fn!())?),
            ],
        )
        .context(current_fn!())?;

        let (sponsor_account, reservation_id, gas_coins) = reserve_gas(NANOS_PER_IOTA * 2, 10)
            .await
            .context(current_fn!())?;
        let ref_gas_price = get_ref_gas_price(&iota_client)
            .await
            .context(current_fn!())?;

        let tx_data = construct_sponsored_tx_data(
            sender,
            gas_coins,
            pt,
            GAS_BUDGET,
            ref_gas_price,
            sponsor_account,
        );

        let signer = sender_key_pair;
        let tx = Transaction::from_data_and_signer(tx_data, vec![&signer]);

        let response = execute_tx(tx, reservation_id)
            .await
            .context(current_fn!())?;

        handle_error_execute_tx(response).context(current_fn!())?;

        Ok(())
    }

    pub async fn update_administrative_metadata(
        &self,
        activation_key: String,
        private_metadata: String,
        public_metadata: String,
        sender: IotaAddress,
        sender_key_pair: IotaKeyPair,
    ) -> Result<(), HospitalError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            String::from("update_administrative_metadata"),
            self.decmed_package.package_id,
            self.decmed_package.module_hospital_personnel.clone(),
            vec![],
            vec![
                CallArg::Pure(bcs::to_bytes(&activation_key).context(current_fn!())?),
                self.construct_address_id_object_call_arg(false),
                self.construct_hospital_personnel_id_account_object_call_arg(true),
                CallArg::Pure(bcs::to_bytes(&private_metadata).context(current_fn!())?),
                CallArg::Pure(bcs::to_bytes(&public_metadata).context(current_fn!())?),
            ],
        )
        .context(current_fn!())?;
        let (sponsor_account, reservation_id, gas_coins) = reserve_gas(NANOS_PER_IOTA * 2, 10)
            .await
            .context(current_fn!())?;
        let ref_gas_price = get_ref_gas_price(&iota_client)
            .await
            .context(current_fn!())?;

        let tx_data = construct_sponsored_tx_data(
            sender,
            gas_coins,
            pt,
            GAS_BUDGET,
            ref_gas_price,
            sponsor_account,
        );

        let signer = sender_key_pair;
        let tx = Transaction::from_data_and_signer(tx_data, vec![&signer]);

        let response = execute_tx(tx, reservation_id)
            .await
            .context(current_fn!())?;

        handle_error_execute_tx(response).context(current_fn!())?;

        Ok(())
    }

    pub async fn use_activation_key(
        &self,
        activation_key: String,
        hospital_id: String,
        personnel_id: String,
        sender: IotaAddress,
        sender_key_pair: IotaKeyPair,
    ) -> Result<(), HospitalError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            String::from("use_activation_key"),
            self.decmed_package.package_id,
            self.decmed_package.module_hospital_personnel.clone(),
            vec![],
            vec![
                CallArg::Pure(bcs::to_bytes(&activation_key).context(current_fn!())?),
                CallArg::Pure(bcs::to_bytes(&hospital_id).context(current_fn!())?),
                self.construct_hospital_personnel_id_account_object_call_arg(true),
                CallArg::Pure(bcs::to_bytes(&personnel_id).context(current_fn!())?),
            ],
        )
        .context(current_fn!())?;
        let (sponsor_account, reservation_id, gas_coins) = reserve_gas(NANOS_PER_IOTA, 10)
            .await
            .context(current_fn!())?;
        let ref_gas_price = get_ref_gas_price(&iota_client)
            .await
            .context(current_fn!())?;

        let tx_data = construct_sponsored_tx_data(
            sender,
            gas_coins,
            pt,
            GAS_BUDGET,
            ref_gas_price,
            sponsor_account,
        );

        let signer = sender_key_pair;
        let tx = Transaction::from_data_and_signer(tx_data, vec![&signer]);

        let response = execute_tx(tx, reservation_id)
            .await
            .context(current_fn!())?;

        handle_error_execute_tx(response).context(current_fn!())?;

        Ok(())
    }
}
