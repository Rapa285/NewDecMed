use std::fmt::Debug;

use iota_json_rpc_types::{IotaObjectRef, IotaTransactionBlockEffects};
use iota_types::{
    base_types::{IotaAddress, ObjectID},
    Identifier,
};
use r2d2::Pool;
use redis::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::move_call::MoveCall;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AuthRole {
    AdministrativePersonnel,
    MedicalPersonnel,
    Patient,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MoveHospitalPersonnelRole {
    Admin,
    AdministrativePersonnel,
    MedicalPersonnel,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ReencryptionPurposeType {
    Read,
    Update,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AccessKeys {
    pub enc_data_pre_secret_key_seed: String,
    pub k_frag: String,
    pub data_pre_public_key: String,
    pub data_pre_secret_key_seed_capsule: String,
    pub patient_pre_public_key: String,
    pub signer_pre_public_key: String,
}

pub struct AppState {
    pub global_admin_iota_address: String,
    pub global_admin_iota_key_pair: String,
    pub jwt_ecdsa_key_pair: String,
    pub jwt_ecdsa_pub_key: String,
    pub move_call: MoveCall,
    pub proxy_iota_address: String,
    pub proxy_iota_key_pair: String,
    pub redis_pool: Pool<Client>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthenticateHandlerPayload {
    pub signature: String,
    pub iota_address: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthenticateHandlerResponse {
    pub access_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientMedicalMetadata {
    pub capsule: String,
    pub enc_data: String,
    pub enc_key_and_nonce: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrentUser {
    pub iota_address: String,
    pub purpose: ReencryptionPurposeType,
    pub role: AuthRole,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DecmedPackage {
    pub package_id: ObjectID,
    pub module_admin: Identifier,
    pub module_proxy: Identifier,

    pub address_id_object_id: ObjectID,
    pub address_id_object_version: u64,
    pub hospital_id_metadata_object_id: ObjectID,
    pub hospital_id_metadata_object_version: u64,
    pub hospital_personnel_id_account_object_id: ObjectID,
    pub hospital_personnel_id_account_object_version: u64,
    pub patient_id_account_object_id: ObjectID,
    pub patient_id_account_object_version: u64,

    pub global_admin_cap_id: ObjectID,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub status_code: u16,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ExecuteTxResponse {
    pub effects: Option<IotaTransactionBlockEffects>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GenerateAndRegisterProxyAddress {
    pub iota_address: String,
    pub iota_keypair: String,
    pub seed_words: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GenerateJwtHandlerResponse {
    pub public_key: String,
    pub secret_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetNonceHandlerPayload {
    pub iota_address: String, // hex string
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GenerateSignatureHandlerPayload {
    pub iota_keypair: String,
    pub nonce: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JwtClaims {
    pub role: AuthRole,
    pub purpose: ReencryptionPurposeType,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ReserveGasResponse {
    pub error: Option<String>,
    pub result: Option<ReserveGasResult>,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ReserveGasResult {
    pub gas_coins: Vec<IotaObjectRef>,
    pub reservation_id: u64,
    pub sponsor_address: IotaAddress,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HandlerCreateMedicalRecordPayload {
    pub medical_metadata: String,
    pub patient_iota_address: String,
}

#[derive(Debug, Deserialize)]
pub struct HandlerGetAdministrativeDataQueryParams {
    pub patient_iota_address: String,
}

#[derive(Debug, Deserialize)]
pub struct HandlerGetMedicalRecordQueryParams {
    #[serde(deserialize_with = "crate::utils::Utils::empty_string_as_none")]
    pub index: Option<u64>,
    pub patient_iota_address: String,
}

#[derive(Debug, Deserialize)]
pub struct HandlerGetMedicalRecordUpdateQueryParams {
    #[serde(deserialize_with = "crate::utils::Utils::empty_string_as_none")]
    pub index: Option<u64>,
    pub patient_iota_address: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HandlerStoreKeysPayload {
    pub enc_data_pre_secret_key_seed: String,
    pub hospital_personnel_iota_address: String,
    pub k_frag: String,
    pub data_pre_public_key: String,
    pub data_pre_secret_key_seed_capsule: String,
    pub patient_iota_address: String,
    pub patient_pre_public_key: String,
    pub signature: String,
    pub signer_pre_public_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HandlerUpdateMedicalRecordPayload {
    pub medical_metadata: String,
    pub patient_iota_address: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MedicalMetadata {
    pub capsule: String,
    pub cid: String,
    pub created_at: String,
    pub enc_key_and_nonce: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MovePatientAdministrativeMetadata {
    pub private_metadata: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MovePatientMedicalMetadata {
    pub index: u64,
    pub metadata: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PatientPrivateAdministrativeMetadata {
    pub capsule: String,
    pub enc_data: String,
    pub enc_key_nonce: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SuccessResponse<T>
where
    T: Debug,
{
    pub data: T,
    pub status_code: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UtilIpfsAddResponse {
    pub allocations: Vec<String>,
    pub cid: String,
    pub name: String,
    pub size: u64,
}
