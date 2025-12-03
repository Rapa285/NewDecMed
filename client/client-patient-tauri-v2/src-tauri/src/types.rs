use std::fmt;

use iota_json_rpc_types::{IotaObjectRef, IotaTransactionBlockEffects};
use iota_types::{
    base_types::{IotaAddress, ObjectID},
    Identifier,
};
use keyring::Entry;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::move_call::MoveCall;

// Enum.

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum HospitalPersonnelRole {
    Admin,
    AdministrativePersonnel,
    MedicalPersonnel,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum MedicalDataMainCategory {
    Category1,
    Category2,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum MedicalDataSubCategory {
    SubCategory1,
    SubCategory2,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum MoveHospitalPersonnelAccessDataType {
    Administrative,
    Medical,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum MoveHospitalPersonnelAccessType {
    Read,
    Update,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ResponseStatus {
    Error,
    Success,
}

// Struct

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdministrativeData {
    pub private: PrivateAdministrativeData,
}

pub struct AppState {
    pub administrative_data: Option<AdministrativeData>,
    pub auth_state: AuthState,
    pub keys_entry: Entry,
    pub move_call: MoveCall,
    pub scan_state: ScanState,
    pub signin_state: SignInState,
    pub signup_state: SignUpState,
}

#[derive(Deserialize, Serialize)]
pub struct AuthState {
    pub is_registered: bool,
    pub role: Option<HospitalPersonnelRole>,
    pub session_pin: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CommandGetMedicalRecordsResponseData {
    pub cid: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub index: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CommandGetProfileResponse {
    pub id: String,
    #[serde(rename = "idHash")]
    pub id_hash: String,
    #[serde(rename = "iotaAddress")]
    pub iota_address: String,
    pub name: Option<String>,
    #[serde(rename = "prePublicKey")]
    pub pre_public_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CommandProcessQrResponse {
    #[serde(rename = "hospitalPersonnelHospitalName")]
    pub hospital_personnel_hospital_name: String,
    #[serde(rename = "hospitalPersonnelName")]
    pub hospital_personnel_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CommandUpdateProfileInput {
    pub name: String,
    #[serde(rename = "birthPlace")]
    pub birth_place: String,
    #[serde(rename = "dateOfBirth")]
    pub date_of_birth: String,
    pub gender: String,
    pub religion: String,
    pub education: String,
    pub occupation: String,
    #[serde(rename = "maritalStatus")]
    pub marital_status: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DecmedPackage {
    pub package_id: ObjectID,
    pub module_admin: Identifier,
    pub module_patient: Identifier,

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

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ExecuteTxResponse {
    pub effects: Option<IotaTransactionBlockEffects>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KeyNonce {
    pub key: String,
    pub nonce: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KeysEntry {
    pub activation_key: Option<String>,
    pub admin_address: Option<String>,
    pub admin_secret_key: Option<String>,
    pub id: Option<String>,
    pub iota_address: Option<String>,
    pub iota_key_pair: Option<String>,
    pub iota_nonce: Option<String>,
    pub pre_nonce: Option<String>,
    pub pre_public_key: Option<String>,
    pub pre_secret_key: Option<String>,
    pub proxy_jwt: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HospitalPersonnelPublicAdministrativeData {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MedicalData {
    pub anamnesis: String,
    pub physical_check: String,
    pub psychological_check: String,
    pub diagnose: String,
    pub therapy: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MedicalMetadata {
    pub capsule: String,
    pub cid: String,
    pub created_at: String,
    pub enc_key_and_nonce: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MovePatientAccessLog {
    pub access_data_type: Vec<MoveHospitalPersonnelAccessDataType>,
    pub access_type: MoveHospitalPersonnelAccessType,
    pub date: String,
    pub exp_dur: u64,
    pub hospital_metadata: MoveHospitalMetadata,
    pub hospital_personnel_address: IotaAddress,
    pub hospital_personnel_metadata: String,
    pub index: u64,
    pub is_revoked: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MoveCreateAccessData {
    pub access_token: String,
    pub patient_iota_address: String,
    pub patient_name: String,
    pub patient_pre_public_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MoveCreateAccessMetadata {
    pub capsule: String,
    pub enc_data: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MoveHospitalMetadata {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MoveHospitalPersonnelPublicAdministrativeData {
    pub hospital_name: Option<String>,
    pub name: Option<String>,
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PrivateAdministrativeData {
    pub id: String,
    pub name: Option<String>,
    pub birth_place: Option<String>,
    pub date_of_birth: Option<String>,
    pub gender: Option<String>,
    pub religion: Option<String>,
    pub education: Option<String>,
    pub occupation: Option<String>,
    pub marital_status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PrivateAdministrativeMetadata {
    pub capsule: String,
    pub enc_data: String,
    pub enc_key_nonce: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProxyReencryptionPostKeysResponseData {
    pub access_token_read: String,
    pub access_token_update: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProxyReencryptionErrorResponse {
    pub error: String,
    pub status_code: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProxyReencryptionKeysPayload {
    pub enc_hospital_personnel_pre_secret_key_seed: String,
    pub hospital_personnel_iota_address: String,
    pub hospital_personnel_pre_public_key: String,
    pub hospital_personnel_pre_secret_key_seed_capsule: String,
    pub k_frag: String,
    pub patient_iota_address: String,
    pub patient_pre_public_key: String,
    pub signature: String,
    pub signer_pre_public_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProxyReencryptionNoncePayload {
    pub iota_address: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProxyReencryptionSuccessResponse<T> {
    pub data: T,
    pub status_code: u16,
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
pub struct ScanState {
    pub hospital_personnel_qr_content: Option<String>,
}

pub struct SignInState {
    pub pin: Option<String>,
}

pub struct SignUpState {
    pub pin: Option<String>,
    pub seed_words: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SuccessResponse<T> {
    pub data: T,
    pub status: ResponseStatus,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UtilIpfsAddResponse {
    pub allocations: Vec<String>,
    pub cid: String,
    pub name: String,
    pub size: u64,
}

impl fmt::Display for ProxyReencryptionErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl std::error::Error for ProxyReencryptionErrorResponse {}
