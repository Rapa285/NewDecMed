use iota_json_rpc_types::{IotaObjectRef, IotaTransactionBlockEffects};
use iota_types::{
    base_types::{IotaAddress, ObjectID},
    Identifier,
};
use keyring::Entry;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::move_call::MoveCall;

// Enum

#[derive(Debug, Deserialize, Serialize)]
pub enum ResponseStatus {
    Error,
    Success,
}

// Struct

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DecmedPackage {
    pub package_id: ObjectID,
    pub module_admin: Identifier,
    pub module_hospital_personnel: Identifier,

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

pub struct AppState {
    pub keys_entry: Entry,
    pub move_call: MoveCall,
}

#[derive(Deserialize, Serialize)]
pub struct AuthState {
    pub is_signed_up: bool,
    pub session_pin: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct CommandCreateActivationKeyPayload {
    #[serde(rename = "hospitalId")]
    pub hospital_id: String,
    #[serde(rename = "hospitalName")]
    pub hospital_name: String,
}

#[derive(Deserialize, Serialize)]
pub struct CommandGetHospitalsPayload {
    pub cursor: Option<u64>,
    pub size: Option<u64>,
}

#[derive(Deserialize, Serialize)]
pub struct CommandUpdateActivationKeyPayload {
    #[serde(rename = "hospitalAdminCid")]
    pub hospital_admin_cid: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ExecuteTxResponse {
    pub effects: Option<IotaTransactionBlockEffects>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HospitalAdminMetadata {
    pub activation_key: String,
    pub hospital_admin_cid: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HospitalAdminMetadataEncrypted {
    pub capsule: String,
    pub enc_metadata: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KeyNonce {
    pub key: String,
    pub nonce: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KeysEntry {
    pub admin_address: Option<String>,
    pub admin_secret_key: Option<String>,
    pub admin_pre_seed: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ReserveGasResponse {
    pub error: Option<String>,
    pub result: Option<ReserveGasResult>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MoveHospital {
    pub admin_metadata: String,
    pub hospital_metadata: HospitalMetadata,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HospitalMetadata {
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub struct ReserveGasResult {
    pub gas_coins: Vec<IotaObjectRef>,
    pub reservation_id: u64,
    pub sponsor_address: IotaAddress,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SuccessResponse<T> {
    pub data: T,
    pub status: ResponseStatus,
}
