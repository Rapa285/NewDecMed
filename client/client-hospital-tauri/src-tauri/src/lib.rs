mod activation;
mod admin;
mod administrative_personnel;
mod constants;
mod hospital_error;
mod macros;
mod medical_personnel;
mod move_call;
mod shared_cmds;
mod signin;
mod signout;
mod signup;
mod types;
mod utils;

use constants::{
    DECMED_ADDRESS_ID_OBJECT_ID, DECMED_ADDRESS_ID_OBJECT_VERSION, DECMED_GLOBAL_ADMIN_CAP_ID,
    DECMED_HOSPITAL_ID_METADATA_OBJECT_ID, DECMED_HOSPITAL_ID_METADATA_OBJECT_VERSION,
    DECMED_HOSPITAL_PERSONNEL_ID_ACCOUNT_OBJECT_ID,
    DECMED_HOSPITAL_PERSONNEL_ID_ACCOUNT_OBJECT_VERSION, DECMED_MODULE_ADMIN,
    DECMED_MODULE_HOSPITAL_PERSONNEL, DECMED_PACKAGE_ID, DECMED_PATIENT_ID_ACCOUNT_OBJECT_ID,
    DECMED_PATIENT_ID_ACCOUNT_OBJECT_VERSION,
};
use iota_types::{base_types::ObjectID, Identifier};
use keyring::Entry;
use move_call::MoveCall;
use std::str::FromStr;
use tauri::{async_runtime::Mutex, Manager};
use types::{AppState, AuthState, DecmedPackage, KeysEntry, SignInState, SignUpState};

fn setup(app: &mut tauri::App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let keys_entry = Entry::new("decmed_service_keys", "decmed_user")?;
    let decmed_package = DecmedPackage {
        package_id: ObjectID::from_str(DECMED_PACKAGE_ID)?,
        module_hospital_personnel: Identifier::from_str(DECMED_MODULE_HOSPITAL_PERSONNEL)?,
        module_admin: Identifier::from_str(DECMED_MODULE_ADMIN)?,

        address_id_object_id: ObjectID::from_str(DECMED_ADDRESS_ID_OBJECT_ID)?,
        address_id_object_version: DECMED_ADDRESS_ID_OBJECT_VERSION,
        hospital_id_metadata_object_id: ObjectID::from_str(DECMED_HOSPITAL_ID_METADATA_OBJECT_ID)?,
        hospital_id_metadata_object_version: DECMED_HOSPITAL_ID_METADATA_OBJECT_VERSION,
        hospital_personnel_id_account_object_id: ObjectID::from_str(
            DECMED_HOSPITAL_PERSONNEL_ID_ACCOUNT_OBJECT_ID,
        )?,
        hospital_personnel_id_account_object_version:
            DECMED_HOSPITAL_PERSONNEL_ID_ACCOUNT_OBJECT_VERSION,
        patient_id_account_object_id: ObjectID::from_str(DECMED_PATIENT_ID_ACCOUNT_OBJECT_ID)?,
        patient_id_account_object_version: DECMED_PATIENT_ID_ACCOUNT_OBJECT_VERSION,

        global_admin_cap_id: ObjectID::from_str(DECMED_GLOBAL_ADMIN_CAP_ID)?,
    };
    let new_keys_entry = KeysEntry {
        id: None,
        admin_address: Some(String::from(
            "0x52a65ae806223e49aaff1cf7f670fee87c1767de1d200a661c1fee44a61fc37f",
        )),
        admin_secret_key: Some(String::from(
            "iotaprivkey1qpfc5nqsvs64p40347h0vcdxz3pgfn72uznw4pfvkak59fhpevxs73z6kwn",
        )),
        activation_key: None,
        iota_address: None,
        iota_key_pair: None,
        pre_secret_key: None,
        pre_public_key: None,
        iota_nonce: None,
        pre_nonce: None,
    };
    let signin_state = SignInState { pin: None };
    let signup_state = SignUpState {
        seed_words: None,
        pin: None,
    };
    let auth_state = AuthState {
        is_signed_up: false,
        role: None,
        session_pin: Some("123456".to_string()),
    };
    let move_call = MoveCall {
        decmed_package: decmed_package.clone(),
    };

    match keys_entry.get_secret() {
        Ok(_) => {
            // let new_keys_entry = serde_json::to_vec(&new_keys_entry).unwrap();
            // keys_entry.set_secret(&new_keys_entry).unwrap();
        }
        Err(err @ keyring::Error::NoEntry) => {
            let new_keys_entry = serde_json::to_vec(&new_keys_entry).unwrap();
            keys_entry.set_secret(&new_keys_entry).unwrap();

            println!("{:#?}", err);
        }
        Err(err) => {
            println!("{:#?}", err);
        }
    }

    app.manage(Mutex::new(AppState {
        administrative_data: None,
        auth_state,
        keys_entry,
        move_call,
        signin_state,
        signup_state,
    }));

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_fs::init())
        .setup(setup)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            activation::global_admin_add_activation_key,
            activation::hospital_admin_add_activation_key,
            activation::activate_app,
            signup::generate_mnemonic,
            signup::signup,
            signup::is_signed_up,
            signout::signout,
            signout::reset,
            signin::signin,
            shared_cmds::validate_pin,
            shared_cmds::validate_confirm_pin,
            shared_cmds::get_profile,
            shared_cmds::update_profile,
            shared_cmds::auth_status,
            admin::get_hospital_personnels,
            medical_personnel::new_medical_record,
            medical_personnel::get_medical_record,
            medical_personnel::get_medical_record_update,
            medical_personnel::get_read_access_medical_personnel,
            medical_personnel::get_update_access_medical_personnel,
            medical_personnel::update_medical_record,
            administrative_personnel::get_administrative_data,
            administrative_personnel::get_read_access_administrative_personnel,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
