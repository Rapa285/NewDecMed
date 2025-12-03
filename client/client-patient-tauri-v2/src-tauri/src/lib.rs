mod access;
mod constants;
mod home;
mod macros;
mod move_call;
mod patient_error;
mod scan;
mod shared_cmds;
mod signin;
mod signout;
mod signup;
mod types;
mod utils;

use anyhow::Context;
use constants::{
    DECMED_ADDRESS_ID_OBJECT_ID, DECMED_ADDRESS_ID_OBJECT_VERSION, DECMED_GLOBAL_ADMIN_CAP_ID,
    DECMED_HOSPITAL_ID_METADATA_OBJECT_ID, DECMED_HOSPITAL_ID_METADATA_OBJECT_VERSION,
    DECMED_HOSPITAL_PERSONNEL_ID_ACCOUNT_OBJECT_ID,
    DECMED_HOSPITAL_PERSONNEL_ID_ACCOUNT_OBJECT_VERSION, DECMED_MODULE_ADMIN,
    DECMED_MODULE_PATIENT, DECMED_PACKAGE_ID, DECMED_PATIENT_ID_ACCOUNT_OBJECT_ID,
    DECMED_PATIENT_ID_ACCOUNT_OBJECT_VERSION,
};
use iota_types::{base_types::ObjectID, Identifier};
use keyring::Entry;
use move_call::MoveCall;
use std::str::FromStr;
use tauri::{async_runtime::Mutex, Manager};
use types::{AppState, AuthState, DecmedPackage, KeysEntry, ScanState, SignInState, SignUpState};

fn setup(app: &mut tauri::App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // #[cfg(target_os = "android")]
    // app.get_webview_window("main")
    //     .unwrap()
    //     .with_webview(|webview| {
    //         webview.jni_handle().exec(|env, context, _webview| {
    //             use tauri::wry::prelude::JObject;
    //             let loader = env
    //                 .call_method(
    //                     context,
    //                     "getClassLoader",
    //                     "()Ljava/lang/ClassLoader;",
    //                     &[],
    //                 )
    //                 .unwrap();

    //             rustls_platform_verifier::android::init_with_refs(
    //                 env.get_java_vm().unwrap(),
    //                 env.new_global_ref(context).unwrap(),
    //                 env.new_global_ref(JObject::try_from(loader).unwrap())
    //                     .unwrap(),
    //             );
    //         });
    //     })?;
    //
    //
    //

    let keys_entry =
        Entry::new("decmed_patient_service_keys", "decmed_patient").context(current_fn!())?;

    let decmed_package = DecmedPackage {
        package_id: ObjectID::from_str(DECMED_PACKAGE_ID)?,
        module_admin: Identifier::from_str(DECMED_MODULE_ADMIN)?,
        module_patient: Identifier::from_str(DECMED_MODULE_PATIENT)?,

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
        admin_secret_key: Some(String::from(
            "iotaprivkey1qpfc5nqsvs64p40347h0vcdxz3pgfn72uznw4pfvkak59fhpevxs73z6kwn",
        )),
        admin_address: Some(String::from(
            "0x52a65ae806223e49aaff1cf7f670fee87c1767de1d200a661c1fee44a61fc37f",
        )),
        activation_key: None,
        id: None,
        iota_address: None,
        iota_key_pair: None,
        iota_nonce: None,
        pre_nonce: None,
        proxy_jwt: None,
        pre_public_key: None,
        pre_secret_key: None,
    };
    let signin_state = SignInState { pin: None };
    let signup_state = SignUpState {
        seed_words: None,
        pin: None,
    };
    let auth_state = AuthState {
        is_registered: false,
        role: None,
        session_pin: Some("123456".to_string()),
    };
    let move_call = MoveCall { decmed_package };
    let scan_state = ScanState {
        hospital_personnel_qr_content: None,
    };

    match keys_entry.get_secret() {
        Ok(_) => {
            // let new_keys_entry = serde_json::to_vec(&new_keys_entry).context(current_fn!())?;
            // keys_entry
            //     .set_secret(&new_keys_entry)
            //     .context(current_fn!())?;
        }
        Err(err @ keyring::Error::NoEntry) => {
            let new_keys_entry = serde_json::to_vec(&new_keys_entry).context(current_fn!())?;
            keys_entry
                .set_secret(&new_keys_entry)
                .context(current_fn!())?;

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
        scan_state,
        signin_state,
        signup_state,
    }));

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(setup)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            signin::signin,
            signup::generate_mnemonic,
            signup::signup,
            signout::signout,
            shared_cmds::auth_status,
            shared_cmds::validate_pin,
            shared_cmds::validate_confirm_pin,
            shared_cmds::is_session_pin_exist,
            shared_cmds::get_profile,
            shared_cmds::validate_seed_words,
            shared_cmds::update_profile,
            scan::process_qr,
            scan::create_access,
            home::get_medical_records,
            home::get_medical_record,
            access::revoke_access,
            access::get_access_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
