use std::str::FromStr;

use anyhow::Context;
use axum::http::StatusCode;
use iota_sdk::IotaClient;
use iota_types::{
    base_types::{IotaAddress, ObjectID},
    crypto::IotaKeyPair,
    gas_coin::NANOS_PER_IOTA,
    transaction::{CallArg, Transaction},
    Identifier,
};
use move_core_types::account_address::AccountAddress;

use crate::{
    constants::{DECMED_MODULE_SHARED, DECMED_PACKAGE_ID, GAS_BUDGET},
    current_fn,
    proxy_error::{ProxyError, ResultExt},
    types::{
        DecmedPackage, MoveHospitalPersonnelRole, MovePatientAdministrativeMetadata,
        MovePatientMedicalMetadata,
    },
    utils::Utils,
};

pub struct MoveCall {
    pub decmed_package: DecmedPackage,
}

impl MoveCall {
    pub fn construct_address_id_object_call_arg(&self, mutable: bool) -> CallArg {
        Utils::construct_shared_object_call_arg(
            self.decmed_package.address_id_object_id,
            self.decmed_package.address_id_object_version,
            mutable,
        )
    }

    pub fn construct_clock_call_arg(&self) -> CallArg {
        Utils::construct_shared_object_call_arg(ObjectID::from_str("0x6").unwrap(), 1, false)
    }

    pub async fn construct_global_admin_cap(&self) -> Result<CallArg, ProxyError> {
        let iota_client = Utils::get_iota_client().await.context(current_fn!())?;
        Ok(Utils::construct_capability_call_arg(
            &iota_client,
            self.decmed_package.global_admin_cap_id,
        )
        .await
        .context(current_fn!())?)
    }

    pub fn construct_hospital_personnel_id_account_object_call_arg(
        &self,
        mutable: bool,
    ) -> CallArg {
        Utils::construct_shared_object_call_arg(
            self.decmed_package.hospital_personnel_id_account_object_id,
            self.decmed_package
                .hospital_personnel_id_account_object_version,
            mutable,
        )
    }

    pub fn construct_patient_id_account_object_call_arg(&self, mutable: bool) -> CallArg {
        Utils::construct_shared_object_call_arg(
            self.decmed_package.patient_id_account_object_id,
            self.decmed_package.patient_id_account_object_version,
            mutable,
        )
    }

    pub async fn construct_proxy_cap(
        &self,
        iota_client: &IotaClient,
        module: Identifier,
        package_id: AccountAddress,
        proxy_iota_address: IotaAddress,
    ) -> Result<CallArg, ProxyError> {
        let cap = Utils::get_proxy_cap(iota_client, module, package_id, proxy_iota_address)
            .await
            .context(current_fn!())?;

        Ok(
            Utils::construct_capability_call_arg(iota_client, cap.object_id)
                .await
                .context(current_fn!())?,
        )
    }

    pub async fn create_capability(
        &self,
        proxy_iota_address: &IotaAddress,
        sender: IotaAddress,
        sender_key_pair: IotaKeyPair,
    ) -> Result<(), ProxyError> {
        let iota_client = Utils::get_iota_client().await.context(current_fn!())?;
        let pt = Utils::construct_pt(
            "create_capability",
            self.decmed_package.package_id,
            self.decmed_package.module_proxy.clone(),
            vec![],
            vec![
                CallArg::Pure(bcs::to_bytes(proxy_iota_address).context(current_fn!())?),
                self.construct_global_admin_cap()
                    .await
                    .context(current_fn!())?,
            ],
        )
        .context(current_fn!())?;

        let (sponsor_account, reservation_id, gas_coins) =
            Utils::reserve_gas(NANOS_PER_IOTA * 2, 10)
                .await
                .context(current_fn!())?;
        let ref_gas_price = Utils::get_ref_gas_price(&iota_client)
            .await
            .context(current_fn!())?;

        let tx_data = Utils::construct_sponsored_tx_data(
            sender,
            gas_coins,
            pt,
            GAS_BUDGET,
            ref_gas_price,
            sponsor_account,
        );

        let signer = sender_key_pair;
        let tx = Transaction::from_data_and_signer(tx_data, vec![&signer]);

        let response = Utils::execute_tx(tx, reservation_id)
            .await
            .context(current_fn!())?;

        Utils::handle_error_execute_tx(response).context(current_fn!())?;

        Ok(())
    }

    pub async fn create_medical_record(
        &self,
        hospital_personnel_address: &IotaAddress,
        metadata: String,
        patient_address: &IotaAddress,
        sender: IotaAddress,
        sender_key_pair: IotaKeyPair,
    ) -> Result<(), ProxyError> {
        let iota_client = Utils::get_iota_client().await.context(current_fn!())?;
        let pt = Utils::construct_pt(
            "create_medical_record",
            self.decmed_package.package_id,
            self.decmed_package.module_proxy.clone(),
            vec![],
            vec![
                self.construct_address_id_object_call_arg(false),
                self.construct_clock_call_arg(),
                CallArg::Pure(bcs::to_bytes(hospital_personnel_address).context(current_fn!())?),
                self.construct_hospital_personnel_id_account_object_call_arg(true),
                CallArg::Pure(bcs::to_bytes(&metadata).context(current_fn!())?),
                CallArg::Pure(bcs::to_bytes(patient_address).context(current_fn!())?),
                self.construct_patient_id_account_object_call_arg(true),
                self.construct_proxy_cap(
                    &iota_client,
                    Identifier::from_str(DECMED_MODULE_SHARED).context(current_fn!())?,
                    AccountAddress::from_str(DECMED_PACKAGE_ID).context(current_fn!())?,
                    sender,
                )
                .await
                .context(current_fn!())?,
            ],
        )
        .context(current_fn!())?;

        let (sponsor_account, reservation_id, gas_coins) =
            Utils::reserve_gas(NANOS_PER_IOTA * 2, 10)
                .await
                .context(current_fn!())?;
        let ref_gas_price = Utils::get_ref_gas_price(&iota_client)
            .await
            .context(current_fn!())?;

        let tx_data = Utils::construct_sponsored_tx_data(
            sender,
            gas_coins,
            pt,
            GAS_BUDGET,
            ref_gas_price,
            sponsor_account,
        );

        let signer = sender_key_pair;
        let tx = Transaction::from_data_and_signer(tx_data, vec![&signer]);

        let response = Utils::execute_tx(tx, reservation_id)
            .await
            .context(current_fn!())?;

        Utils::handle_error_execute_tx(response).context(current_fn!())?;

        Ok(())
    }

    pub async fn get_administrative_data(
        &self,
        hospital_personnel_address: &IotaAddress,
        patient_address: &IotaAddress,
        sender: IotaAddress,
    ) -> Result<MovePatientAdministrativeMetadata, ProxyError> {
        let iota_client = Utils::get_iota_client().await.context(current_fn!())?;
        let pt = Utils::construct_pt(
            "get_administrative_data",
            self.decmed_package.package_id,
            self.decmed_package.module_proxy.clone(),
            vec![],
            vec![
                self.construct_address_id_object_call_arg(false),
                self.construct_clock_call_arg(),
                CallArg::Pure(bcs::to_bytes(hospital_personnel_address).context(current_fn!())?),
                self.construct_hospital_personnel_id_account_object_call_arg(true),
                CallArg::Pure(bcs::to_bytes(patient_address).context(current_fn!())?),
                self.construct_patient_id_account_object_call_arg(false),
                self.construct_proxy_cap(
                    &iota_client,
                    Identifier::from_str(DECMED_MODULE_SHARED).context(current_fn!())?,
                    AccountAddress::from_str(DECMED_PACKAGE_ID).context(current_fn!())?,
                    sender,
                )
                .await
                .context(current_fn!())?,
            ],
        )
        .context(current_fn!())?;

        let response = Utils::move_call_read_only(sender, &iota_client, pt)
            .await
            .context(current_fn!())?;

        Utils::handle_error_move_call_read_only(response.clone())
            .context(current_fn!())
            .code(StatusCode::UNAUTHORIZED)?;

        let role: MovePatientAdministrativeMetadata =
            Utils::parse_move_read_only_result(response, 0).context(current_fn!())?;

        Ok(role)
    }

    pub async fn get_hospital_personnel_role(
        &self,
        hospital_personnel_address: &IotaAddress,
        sender: IotaAddress,
    ) -> Result<MoveHospitalPersonnelRole, ProxyError> {
        let iota_client = Utils::get_iota_client().await.context(current_fn!())?;
        let pt = Utils::construct_pt(
            "get_hospital_personnel_role",
            self.decmed_package.package_id,
            self.decmed_package.module_proxy.clone(),
            vec![],
            vec![
                self.construct_address_id_object_call_arg(false),
                self.construct_hospital_personnel_id_account_object_call_arg(false),
                CallArg::Pure(bcs::to_bytes(hospital_personnel_address).context(current_fn!())?),
                self.construct_proxy_cap(
                    &iota_client,
                    Identifier::from_str(DECMED_MODULE_SHARED).context(current_fn!())?,
                    AccountAddress::from_str(DECMED_PACKAGE_ID).context(current_fn!())?,
                    sender,
                )
                .await
                .context(current_fn!())?,
            ],
        )
        .context(current_fn!())?;

        let response = Utils::move_call_read_only(sender, &iota_client, pt)
            .await
            .context(current_fn!())?;

        Utils::handle_error_move_call_read_only(response.clone())
            .context(current_fn!())
            .code(StatusCode::UNAUTHORIZED)?;

        let role: MoveHospitalPersonnelRole =
            Utils::parse_move_read_only_result(response, 0).context(current_fn!())?;

        Ok(role)
    }

    /// ## Returns:
    /// 1: prev_index
    /// 2: next_index
    pub async fn get_medical_record(
        &self,
        hospital_personnel_address: &IotaAddress,
        index: u64,
        patient_address: &IotaAddress,
        sender: IotaAddress,
    ) -> Result<
        (
            MovePatientMedicalMetadata,
            MovePatientAdministrativeMetadata,
            u64,
            Option<u64>,
            Option<u64>,
        ),
        ProxyError,
    > {
        let iota_client = Utils::get_iota_client().await.context(current_fn!())?;
        let pt = Utils::construct_pt(
            "get_medical_record",
            self.decmed_package.package_id,
            self.decmed_package.module_proxy.clone(),
            vec![],
            vec![
                self.construct_address_id_object_call_arg(false),
                self.construct_clock_call_arg(),
                CallArg::Pure(bcs::to_bytes(hospital_personnel_address).context(current_fn!())?),
                self.construct_hospital_personnel_id_account_object_call_arg(true),
                CallArg::Pure(bcs::to_bytes(&index).context(current_fn!())?),
                CallArg::Pure(bcs::to_bytes(patient_address).context(current_fn!())?),
                self.construct_patient_id_account_object_call_arg(false),
                self.construct_proxy_cap(
                    &iota_client,
                    Identifier::from_str(DECMED_MODULE_SHARED).context(current_fn!())?,
                    AccountAddress::from_str(DECMED_PACKAGE_ID).context(current_fn!())?,
                    sender,
                )
                .await
                .context(current_fn!())?,
            ],
        )
        .context(current_fn!())?;

        let response = Utils::move_call_read_only(sender, &iota_client, pt)
            .await
            .context(current_fn!())?;

        Utils::handle_error_move_call_read_only(response.clone())
            .context(current_fn!())
            .code(StatusCode::UNAUTHORIZED)?;

        let medical_metadata: MovePatientMedicalMetadata =
            Utils::parse_move_read_only_result(response.clone(), 0).context(current_fn!())?;
        let administrative_metadata: MovePatientAdministrativeMetadata =
            Utils::parse_move_read_only_result(response.clone(), 1).context(current_fn!())?;
        let current_index: u64 =
            Utils::parse_move_read_only_result(response.clone(), 2).context(current_fn!())?;
        let prev_index: Option<u64> =
            Utils::parse_move_read_only_result(response.clone(), 3).context(current_fn!())?;
        let next_index: Option<u64> =
            Utils::parse_move_read_only_result(response, 4).context(current_fn!())?;

        Ok((
            medical_metadata,
            administrative_metadata,
            current_index,
            prev_index,
            next_index,
        ))
    }

    pub async fn get_medical_record_update(
        &self,
        hospital_personnel_address: &IotaAddress,
        index: u64,
        patient_address: &IotaAddress,
        sender: IotaAddress,
    ) -> Result<
        (
            MovePatientMedicalMetadata,
            MovePatientAdministrativeMetadata,
        ),
        ProxyError,
    > {
        let iota_client = Utils::get_iota_client().await.context(current_fn!())?;
        let pt = Utils::construct_pt(
            "get_medical_record_update",
            self.decmed_package.package_id,
            self.decmed_package.module_proxy.clone(),
            vec![],
            vec![
                self.construct_address_id_object_call_arg(false),
                self.construct_clock_call_arg(),
                CallArg::Pure(bcs::to_bytes(hospital_personnel_address).context(current_fn!())?),
                self.construct_hospital_personnel_id_account_object_call_arg(true),
                CallArg::Pure(bcs::to_bytes(&index).context(current_fn!())?),
                CallArg::Pure(bcs::to_bytes(patient_address).context(current_fn!())?),
                self.construct_patient_id_account_object_call_arg(false),
                self.construct_proxy_cap(
                    &iota_client,
                    Identifier::from_str(DECMED_MODULE_SHARED).context(current_fn!())?,
                    AccountAddress::from_str(DECMED_PACKAGE_ID).context(current_fn!())?,
                    sender,
                )
                .await
                .context(current_fn!())?,
            ],
        )
        .context(current_fn!())?;

        let response = Utils::move_call_read_only(sender, &iota_client, pt)
            .await
            .context(current_fn!())?;

        Utils::handle_error_move_call_read_only(response.clone())
            .context(current_fn!())
            .code(StatusCode::UNAUTHORIZED)?;

        let medical_metadata: MovePatientMedicalMetadata =
            Utils::parse_move_read_only_result(response.clone(), 0).context(current_fn!())?;
        let administrative_metadata: MovePatientAdministrativeMetadata =
            Utils::parse_move_read_only_result(response.clone(), 1).context(current_fn!())?;

        Ok((medical_metadata, administrative_metadata))
    }

    pub async fn is_patient_registered(
        &self,
        patient_address: &IotaAddress,
        sender: IotaAddress,
    ) -> Result<bool, ProxyError> {
        let iota_client = Utils::get_iota_client().await.context(current_fn!())?;
        let pt = Utils::construct_pt(
            "is_patient_registered",
            self.decmed_package.package_id,
            self.decmed_package.module_proxy.clone(),
            vec![],
            vec![
                self.construct_address_id_object_call_arg(false),
                self.construct_patient_id_account_object_call_arg(false),
                CallArg::Pure(bcs::to_bytes(patient_address).context(current_fn!())?),
                self.construct_proxy_cap(
                    &iota_client,
                    Identifier::from_str(DECMED_MODULE_SHARED).context(current_fn!())?,
                    AccountAddress::from_str(DECMED_PACKAGE_ID).context(current_fn!())?,
                    sender,
                )
                .await
                .context(current_fn!())?,
            ],
        )
        .context(current_fn!())?;

        let response = Utils::move_call_read_only(sender, &iota_client, pt)
            .await
            .context(current_fn!())?;

        Utils::handle_error_move_call_read_only(response.clone())
            .context(current_fn!())
            .code(StatusCode::UNAUTHORIZED)?;

        Ok(true)
    }

    pub async fn update_medical_record(
        &self,
        hospital_personnel_address: &IotaAddress,
        metadata: String,
        patient_address: &IotaAddress,
        sender: IotaAddress,
        sender_key_pair: IotaKeyPair,
    ) -> Result<(), ProxyError> {
        let iota_client = Utils::get_iota_client().await.context(current_fn!())?;
        let pt = Utils::construct_pt(
            "update_medical_record",
            self.decmed_package.package_id,
            self.decmed_package.module_proxy.clone(),
            vec![],
            vec![
                self.construct_address_id_object_call_arg(false),
                self.construct_clock_call_arg(),
                CallArg::Pure(bcs::to_bytes(hospital_personnel_address).context(current_fn!())?),
                self.construct_hospital_personnel_id_account_object_call_arg(true),
                CallArg::Pure(bcs::to_bytes(&metadata).context(current_fn!())?),
                CallArg::Pure(bcs::to_bytes(patient_address).context(current_fn!())?),
                self.construct_patient_id_account_object_call_arg(true),
                self.construct_proxy_cap(
                    &iota_client,
                    Identifier::from_str(DECMED_MODULE_SHARED).context(current_fn!())?,
                    AccountAddress::from_str(DECMED_PACKAGE_ID).context(current_fn!())?,
                    sender,
                )
                .await
                .context(current_fn!())?,
            ],
        )
        .context(current_fn!())?;

        let (sponsor_account, reservation_id, gas_coins) =
            Utils::reserve_gas(NANOS_PER_IOTA * 2, 10)
                .await
                .context(current_fn!())?;
        let ref_gas_price = Utils::get_ref_gas_price(&iota_client)
            .await
            .context(current_fn!())?;

        let tx_data = Utils::construct_sponsored_tx_data(
            sender,
            gas_coins,
            pt,
            GAS_BUDGET,
            ref_gas_price,
            sponsor_account,
        );

        let signer = sender_key_pair;
        let tx = Transaction::from_data_and_signer(tx_data, vec![&signer]);

        let response = Utils::execute_tx(tx, reservation_id)
            .await
            .context(current_fn!())?;

        Utils::handle_error_execute_tx(response).context(current_fn!())?;

        Ok(())
    }
}
