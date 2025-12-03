mod activation;
mod client_error;
mod constants;
mod hospital;
mod macros;
mod move_call;
mod types;
mod utils;

use std::str::FromStr;

use iota_types::{base_types::ObjectID, Identifier};
use keyring::Entry;
use tauri::{async_runtime::Mutex, Manager};

use crate::{
    constants::{
        DECMED_ADDRESS_ID_OBJECT_ID, DECMED_ADDRESS_ID_OBJECT_VERSION, DECMED_GLOBAL_ADMIN_CAP_ID,
        DECMED_HOSPITAL_ID_METADATA_OBJECT_ID, DECMED_HOSPITAL_ID_METADATA_OBJECT_VERSION,
        DECMED_HOSPITAL_PERSONNEL_ID_ACCOUNT_OBJECT_ID,
        DECMED_HOSPITAL_PERSONNEL_ID_ACCOUNT_OBJECT_VERSION, DECMED_MODULE_ADMIN,
        DECMED_MODULE_HOSPITAL_PERSONNEL, DECMED_PACKAGE_ID, DECMED_PATIENT_ID_ACCOUNT_OBJECT_ID,
        DECMED_PATIENT_ID_ACCOUNT_OBJECT_VERSION,
    },
    move_call::MoveCall,
    types::{AppState, DecmedPackage, KeysEntry},
};

fn setup(app: &mut tauri::App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let keys_entry = Entry::new("decmed_ministry_keys", "decmed_ministry")?;
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
        admin_address: Some(String::from(
            "0x52a65ae806223e49aaff1cf7f670fee87c1767de1d200a661c1fee44a61fc37f",
        )),
        admin_secret_key: Some(String::from(
            "iotaprivkey1qpfc5nqsvs64p40347h0vcdxz3pgfn72uznw4pfvkak59fhpevxs73z6kwn",
        )),
        admin_pre_seed: String::from("sM5LRtjsf30Gsbmw7sWesgkdrAOzA9F6qMP8xrmXl1o="),
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
        keys_entry,
        move_call,
    }));

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(setup)
        .invoke_handler(tauri::generate_handler![
            activation::create_activation_key,
            activation::generate_pre_seed,
            activation::update_activation_key,
            hospital::get_hospitals,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
