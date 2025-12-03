module decmed::admin;

use decmed::shared::{
    GlobalAdminCap,

    encode_hospital_id,
    encode_hospital_personnel_id,
};

use decmed::std_struct_hospital::{
    new as hospital_new,
    Hospital,
};
use decmed::std_struct_hospital_id_metadata::HospitalIdMetadata;
use decmed::std_struct_hospital_metadata::{
    new as hospital_metadata_new
};
use decmed::std_struct_hospital_personnel_account::{
    new as hospital_personnel_account_new
};
use decmed::std_struct_hospital_personnel_id_account::HospitalPersonnelIdAccount;
use decmed::std_enum_hospital_personnel_role::{
    admin as hospital_personnel_role_admin,
};

use std::string::{String};

// Constants

const EAccountAlreadyRegistered: u64 = 1000;
const EAccountNotFound: u64 = 1001;
const EHospitalAlreadyRegistered: u64 = 1002;
const EHospitalNotFound: u64 = 1003;

// Functions

/// ## Params
/// - `activation_key`: argon_hash(<raw_uuid_v4>@<raw_id>)
/// - `hospital_id`: argon_hash(raw_hospital_id)
/// - `hospital_name`: raw_hospital_name
/// - `id`: argon_hash(raw_id)
entry fun create_activation_key(
    activation_key: String,
    hospital_admin_id: String,
    hospital_admin_metadata: String,
    hospital_id: String,
    hospital_id_metadata: &mut HospitalIdMetadata,
    hospital_name: String,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    _: &GlobalAdminCap,
)
{
    let hospital_id = encode_hospital_id(hospital_id);
    let hospital_id_metadata_table = hospital_id_metadata.borrow_table();
    let hospital_id_metadata_vec = hospital_id_metadata.borrow_vec();
    let hospital_metadata_index = hospital_id_metadata_vec.length();

    assert!(!hospital_id_metadata_table.contains(hospital_id), EHospitalAlreadyRegistered);

    let hospital_metadata = hospital_metadata_new(hospital_name);
    let hospital = hospital_new(hospital_admin_metadata, hospital_metadata);

    let hospital_id_metadata_table = hospital_id_metadata.borrow_mut_table();
    hospital_id_metadata_table.add(hospital_id, hospital_metadata_index);

    let hospital_id_metadata_vec = hospital_id_metadata.borrow_mut_vec();
    hospital_id_metadata_vec.push_back(hospital);

    let hospital_personnel_id = encode_hospital_personnel_id(hospital_id, hospital_admin_id);
    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_mut_table();

    assert!(!hospital_personnel_id_account_table.contains(hospital_personnel_id), EAccountAlreadyRegistered);

    let account = hospital_personnel_account_new(
        option::none(),
        activation_key,
        option::none(),
        option::none(),
        hospital_id,
        false,
        false,
        option::none(),
        hospital_personnel_role_admin(),
    );

    hospital_personnel_id_account_table.add(hospital_personnel_id, account);
}

#[test_only]
public(package) fun create_activation_key_test(
    activation_key: String,
    hospital_admin_id: String,
    hospital_admin_metadata: String,
    hospital_id: String,
    hospital_id_metadata: &mut HospitalIdMetadata,
    hospital_name: String,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    global_admin_cap: &GlobalAdminCap,
)
{
    create_activation_key(
        activation_key,
        hospital_admin_id,
        hospital_admin_metadata,
        hospital_id,
        hospital_id_metadata,
        hospital_name,
        hospital_personnel_id_account,
        global_admin_cap,
    )
}

entry fun get_hospitals(
    cursor: u64,
    hospital_id_metadata: &HospitalIdMetadata,
    size: u64,
    _: &GlobalAdminCap,
): vector<Hospital>
{
    let hospital_id_metadata_vec = hospital_id_metadata.borrow_vec();

    let max_length = hospital_id_metadata_vec.length();
    let size = std::u64::min(10, size);

    let mut hospital = vector::empty<Hospital>();

    if (max_length == 0) {
        return hospital
    };

    let mut curr_index = cursor;
    let end_index = std::u64::min(max_length - 1, cursor + size);


    while (curr_index <= end_index) {
        let hs = hospital_id_metadata_vec.borrow(curr_index);
        hospital.push_back(*hs);

        curr_index = curr_index + 1;
    };

    hospital
}

/// ## Params
/// - `activation_key`: argon_hash(<raw_uuid_v4>@<raw_id>)
/// - `hospital_id`: argon_hash(raw_hospital_id)
/// - `id`: argon_hash(raw_id)
entry fun update_activation_key(
    activation_key: String,
    hospital_admin_id: String,
    hospital_admin_metadata: String,
    hospital_id: String,
    hospital_id_metadata: &mut HospitalIdMetadata,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    _: &GlobalAdminCap,
    _ctx: &TxContext,
)
{
    let hospital_id = encode_hospital_id(hospital_id);
    let hospital_id_metadata_table = hospital_id_metadata.borrow_table();

    assert!(hospital_id_metadata_table.contains(hospital_id), EHospitalNotFound);

    let hospital_id_metadata_index = *hospital_id_metadata_table.borrow(hospital_id);
    let hospital_id_metadata_vec = hospital_id_metadata.borrow_mut_vec();
    let hospital_metadata = hospital_id_metadata_vec.borrow_mut(hospital_id_metadata_index);
    hospital_metadata.set_admin_metadata(hospital_admin_metadata);

    let hospital_personnel_id = encode_hospital_personnel_id(hospital_id, hospital_admin_id);
    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_mut_table();

    assert!(hospital_personnel_id_account_table.contains(hospital_personnel_id), EAccountNotFound);

    let hospital_personnel_account = hospital_personnel_id_account_table.borrow_mut(hospital_personnel_id);

    hospital_personnel_account.set_activation_key(activation_key);
    hospital_personnel_account.set_is_activation_key_used(false);
}
