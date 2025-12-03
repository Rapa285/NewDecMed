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
    patient_error::PatientError,
    types::{
        DecmedPackage, MovePatientAccessLog, MovePatientAdministrativeMetadata,
        MovePatientMedicalMetadata,
    },
    utils::{
        construct_pt, construct_shared_object_call_arg, construct_sponsored_tx_data, execute_tx,
        get_iota_client, get_ref_gas_price, handle_error_execute_tx,
        handle_error_move_call_read_only, move_call_read_only, parse_move_read_only_result,
        reserve_gas,
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

    pub fn construct_patient_id_account_object_call_arg(&self, mutable: bool) -> CallArg {
        construct_shared_object_call_arg(
            self.decmed_package.patient_id_account_object_id,
            self.decmed_package.patient_id_account_object_version,
            mutable,
        )
    }

    pub async fn create_access(
        &self,
        date: String,
        hospital_personnel_address: &IotaAddress,
        metadata: Vec<String>,
        sender: IotaAddress,
        sender_key_pair: IotaKeyPair,
    ) -> Result<(), PatientError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            "create_access".to_string(),
            self.decmed_package.package_id,
            self.decmed_package.module_patient.clone(),
            vec![],
            vec![
                self.construct_address_id_object_call_arg(false),
                self.construct_clock_call_arg(),
                CallArg::Pure(bcs::to_bytes(&date).context(current_fn!())?),
                self.construct_hospital_id_metadata_object_call_arg(false),
                CallArg::Pure(bcs::to_bytes(hospital_personnel_address).context(current_fn!())?),
                self.construct_hospital_personnel_id_account_object_call_arg(true),
                CallArg::Pure(bcs::to_bytes(&metadata).context(current_fn!())?),
                self.construct_patient_id_account_object_call_arg(true),
            ],
        )
        .context(current_fn!())?;

        let (sponsor_account, reservation_id, gas_coins) = reserve_gas(NANOS_PER_IOTA * 5, 10)
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

    pub async fn is_account_registered(&self, sender: IotaAddress) -> Result<bool, PatientError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            String::from("is_account_registered"),
            self.decmed_package.package_id,
            self.decmed_package.module_patient.clone(),
            vec![],
            vec![self.construct_address_id_object_call_arg(false)],
        )
        .context(current_fn!())?;

        let response = move_call_read_only(sender, &iota_client, pt)
            .await
            .context(current_fn!())?;
        handle_error_move_call_read_only(response.clone()).context(current_fn!())?;

        Ok(true)
    }

    pub async fn get_account_info(
        &self,
        sender: IotaAddress,
    ) -> Result<MovePatientAdministrativeMetadata, PatientError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            "get_account_info".to_string(),
            self.decmed_package.package_id,
            self.decmed_package.module_patient.clone(),
            vec![],
            vec![
                self.construct_address_id_object_call_arg(false),
                self.construct_patient_id_account_object_call_arg(false),
            ],
        )
        .context(current_fn!())?;

        let response = move_call_read_only(sender, &iota_client, pt)
            .await
            .context(current_fn!())?;
        handle_error_move_call_read_only(response.clone()).context(current_fn!())?;

        let administrative_data: MovePatientAdministrativeMetadata =
            parse_move_read_only_result(response.clone(), 0).context(current_fn!())?;

        Ok(administrative_data)
    }

    pub async fn get_account_state(
        &self,
        patient_id: String,
        sender: IotaAddress,
    ) -> Result<u64, PatientError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            "get_account_state".to_string(),
            self.decmed_package.package_id,
            self.decmed_package.module_patient.clone(),
            vec![],
            vec![
                CallArg::Pure(bcs::to_bytes(&patient_id).context(current_fn!())?),
                self.construct_patient_id_account_object_call_arg(false),
            ],
        )
        .context(current_fn!())?;

        let response = move_call_read_only(sender, &iota_client, pt)
            .await
            .context(current_fn!())?;
        handle_error_move_call_read_only(response.clone()).context(current_fn!())?;

        let account_state: u64 =
            parse_move_read_only_result(response.clone(), 0).context(current_fn!())?;

        Ok(account_state)
    }

    pub async fn get_hospital_personnel_info(
        &self,
        hospital_personnel_iota_address: &IotaAddress,
        sender: IotaAddress,
    ) -> Result<(String, String), PatientError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            "get_hospital_personnel_info".to_string(),
            self.decmed_package.package_id,
            self.decmed_package.module_patient.clone(),
            vec![],
            vec![
                self.construct_address_id_object_call_arg(false),
                self.construct_hospital_id_metadata_object_call_arg(false),
                CallArg::Pure(
                    bcs::to_bytes(hospital_personnel_iota_address).context(current_fn!())?,
                ),
                self.construct_hospital_personnel_id_account_object_call_arg(false),
            ],
        )
        .context(current_fn!())?;

        let response = move_call_read_only(sender, &iota_client, pt)
            .await
            .context(current_fn!())?;
        handle_error_move_call_read_only(response.clone()).context(current_fn!())?;

        let hospital_personnel_public_administrative_metadata: String =
            parse_move_read_only_result(response.clone(), 0).context(current_fn!())?;
        let hospital_name: String =
            parse_move_read_only_result(response, 1).context(current_fn!())?;

        Ok((
            hospital_personnel_public_administrative_metadata,
            hospital_name,
        ))
    }

    pub async fn get_access_log(
        &self,
        cursor: u64,
        size: u64,
        sender: IotaAddress,
    ) -> Result<Vec<MovePatientAccessLog>, PatientError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            "get_access_log".to_string(),
            self.decmed_package.package_id,
            self.decmed_package.module_patient.clone(),
            vec![],
            vec![
                self.construct_address_id_object_call_arg(false),
                CallArg::Pure(bcs::to_bytes(&cursor).context(current_fn!())?),
                self.construct_patient_id_account_object_call_arg(false),
                CallArg::Pure(bcs::to_bytes(&size).context(current_fn!())?),
            ],
        )
        .context(current_fn!())?;

        let response = move_call_read_only(sender, &iota_client, pt)
            .await
            .context(current_fn!())?;

        handle_error_move_call_read_only(response.clone()).context(current_fn!())?;

        let access_log: Vec<MovePatientAccessLog> =
            parse_move_read_only_result(response.clone(), 0)?;

        Ok(access_log)
    }

    pub async fn get_medical_records(
        &self,
        cursor: u64,
        size: u64,
        sender: IotaAddress,
    ) -> Result<Vec<MovePatientMedicalMetadata>, PatientError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            "get_medical_records".to_string(),
            self.decmed_package.package_id,
            self.decmed_package.module_patient.clone(),
            vec![],
            vec![
                self.construct_address_id_object_call_arg(false),
                CallArg::Pure(bcs::to_bytes(&cursor).context(current_fn!())?),
                self.construct_patient_id_account_object_call_arg(false),
                CallArg::Pure(bcs::to_bytes(&size).context(current_fn!())?),
            ],
        )
        .context(current_fn!())?;

        let response = move_call_read_only(sender, &iota_client, pt)
            .await
            .context(current_fn!())?;

        handle_error_move_call_read_only(response.clone()).context(current_fn!())?;

        let medical_metadata: Vec<MovePatientMedicalMetadata> =
            parse_move_read_only_result(response.clone(), 0)?;

        Ok(medical_metadata)
    }

    pub async fn get_medical_record(
        &self,
        index: u64,
        sender: IotaAddress,
    ) -> Result<MovePatientMedicalMetadata, PatientError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            "get_medical_record".to_string(),
            self.decmed_package.package_id,
            self.decmed_package.module_patient.clone(),
            vec![],
            vec![
                self.construct_address_id_object_call_arg(false),
                CallArg::Pure(bcs::to_bytes(&index).context(current_fn!())?),
                self.construct_patient_id_account_object_call_arg(false),
            ],
        )
        .context(current_fn!())?;

        let response = move_call_read_only(sender, &iota_client, pt)
            .await
            .context(current_fn!())?;
        handle_error_move_call_read_only(response.clone()).context(current_fn!())?;

        let medical_metadata: MovePatientMedicalMetadata =
            parse_move_read_only_result(response.clone(), 0).context(current_fn!())?;

        Ok(medical_metadata)
    }

    pub async fn revoke_access(
        &self,
        hospital_personnel_address: IotaAddress,
        index: u64,
        sender: IotaAddress,
        sender_key_pair: IotaKeyPair,
    ) -> Result<(), PatientError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            String::from("revoke_access"),
            self.decmed_package.package_id,
            self.decmed_package.module_patient.clone(),
            vec![],
            vec![
                self.construct_address_id_object_call_arg(true),
                CallArg::Pure(bcs::to_bytes(&hospital_personnel_address).context(current_fn!())?),
                self.construct_hospital_personnel_id_account_object_call_arg(true),
                CallArg::Pure(bcs::to_bytes(&index).context(current_fn!())?),
                self.construct_patient_id_account_object_call_arg(true),
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
        patient_id: String,
        private_metadata: String,
        sender: IotaAddress,
        sender_key_pair: IotaKeyPair,
    ) -> Result<(), PatientError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            String::from("signup"),
            self.decmed_package.package_id,
            self.decmed_package.module_patient.clone(),
            vec![],
            vec![
                self.construct_address_id_object_call_arg(true),
                CallArg::Pure(bcs::to_bytes(&patient_id).context(current_fn!())?),
                self.construct_patient_id_account_object_call_arg(true),
                CallArg::Pure(bcs::to_bytes(&private_metadata).context(current_fn!())?),
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
        private_metadata: String,
        sender: IotaAddress,
        sender_key_pair: IotaKeyPair,
    ) -> Result<(), PatientError> {
        let iota_client = get_iota_client().await.context(current_fn!())?;
        let pt = construct_pt(
            String::from("update_administrative_metadata"),
            self.decmed_package.package_id,
            self.decmed_package.module_patient.clone(),
            vec![],
            vec![
                self.construct_address_id_object_call_arg(false),
                self.construct_patient_id_account_object_call_arg(true),
                CallArg::Pure(bcs::to_bytes(&private_metadata).context(current_fn!())?),
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
}
