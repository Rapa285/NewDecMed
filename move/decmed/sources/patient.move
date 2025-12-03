module decmed::patient;

use decmed::std_enum_hospital_personnel_role::{
    administrative_personnel as hospital_personnel_role_administrative_personnel,
    medical_personnel as hospital_personnel_role_medical_personnel,
};
use decmed::std_enum_hospital_personnel_access_type::{
    read as hospital_personnel_access_type_read,
    update as hospital_personnel_access_type_update,
};
use decmed::std_enum_hospital_personnel_access_data_type::{
    HospitalPersonnelAccessDataType,
    administrative as hospital_personnel_access_data_type_administrative,
    medical as  hospital_personnel_access_data_type_medical,
};
use decmed::shared::{
    encode_patient_id,
};
use decmed::std_struct_address_id::AddressId;
use decmed::std_struct_hospital_personnel_access_data::{
    HospitalPersonnelAccessData,
    new as hospital_personnel_access_data_new,
};
use decmed::std_struct_hospital_id_metadata::HospitalIdMetadata;
use decmed::std_struct_hospital_personnel_id_account::HospitalPersonnelIdAccount;
use decmed::std_struct_patient_access_log::{
    PatientAccessLog,
    new as patient_access_log_new,
};
use decmed::std_struct_patient_account::{
    new as patient_account_new,
};
use decmed::std_struct_patient_administrative_metadata::{
    PatientAdministrativeMetadata,
    new as patient_administrative_metadata_new,
};
use decmed::std_struct_patient_id_account::PatientIdAccount;
use decmed::std_struct_patient_medical_metadata::PatientMedicalMetadata;

use iota::clock::Clock;
use iota::table_vec;

use std::string::String;

// Constants

const EAccountAlreadyRegistered: u64 = 3000;
const EAddressAlreadyRegistered: u64 = 3001;
const EAccountNotFound: u64 = 3002;
const EAddressNotFound: u64 = 3003;
const EHospitalPersonnelNotFound: u64 = 3004;
const EInvalidMetadataLength: u64 = 3005;

// Enums

// Structs

// Functions

/// ## Params:
/// - `metadata`: vector<Base64 encoded>
///     - length = 1 for administrative
///     - length = 2 for medical
///         - 0: read
///         - 1: update
entry fun create_access(
    address_id: &AddressId,
    clock: &Clock,
    date: String,
    hospital_id_metadata: &HospitalIdMetadata,
    hospital_personnel_address: address,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    metadata: vector<String>,
    patient_id_account: &mut PatientIdAccount,
    ctx: &TxContext,
)
{
    let address_id_table = address_id.borrow_table();
    let patient_id = *address_id_table.borrow(ctx.sender());
    let patient_id_account_table = patient_id_account.borrow_mut_table();
    let patient_account = patient_id_account_table.borrow_mut(patient_id);
    let patient_access_log = patient_account.borrow_mut_access_log();

    assert!(address_id_table.contains(hospital_personnel_address), EHospitalPersonnelNotFound);

    let hospital_personnel_id = *address_id_table.borrow(hospital_personnel_address);
    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_mut_table();
    let hospital_personnel_account = hospital_personnel_id_account_table.borrow(hospital_personnel_id);
    let hospital_personnel_role = *hospital_personnel_account.borrow_role();
    let hospital_personnel_administrative_metadata = hospital_personnel_account.borrow_administrative_metadata().borrow();
    let hospital_personnel_administrative_metadata_public = *hospital_personnel_administrative_metadata.borrow_public_metadata();

    let hospital_id_metadata_table = hospital_id_metadata.borrow_table();
    let hospital_id_metadata_vec = hospital_id_metadata.borrow_vec();
    let hospital_index = *hospital_id_metadata_table.borrow(*hospital_personnel_account.borrow_hospital_id());
    let hospital = hospital_id_metadata_vec.borrow(hospital_index);
    let hospital_metadata = hospital.borrow_hospital_metadata();

    let hospital_personnel_account = hospital_personnel_id_account_table.borrow_mut(hospital_personnel_id);
    let hospital_personnel_access = hospital_personnel_account.borrow_mut_access().borrow_mut();


    if (hospital_personnel_role == hospital_personnel_role_administrative_personnel()) {
        let (read_access, access_data_type_read, exp_dur) = create_access_administrative_personnel(clock, metadata);

        let hospital_personnel_read_access = hospital_personnel_access.borrow_mut_read();

        if (hospital_personnel_read_access.contains(&patient_id)) {
            hospital_personnel_read_access.remove(&patient_id);
        };

        hospital_personnel_read_access.insert(patient_id, read_access);

        let patient_access_log_read = patient_access_log_new(
            access_data_type_read,
            hospital_personnel_access_type_read(),
            date,
            exp_dur,
            *hospital_metadata,
            hospital_personnel_address,
            hospital_personnel_administrative_metadata_public,
            patient_access_log.length(),
            false,
        );
        patient_access_log.push_back(patient_access_log_read);
    };

    if (hospital_personnel_role == hospital_personnel_role_medical_personnel()) {
        let (read_access, access_data_type_read,
            update_access, access_data_type_update, exp_dur_read, exp_dur_update) = create_access_medical_personnel(clock, metadata);

        let hospital_personnel_read_access = hospital_personnel_access.borrow_mut_read();
        if (hospital_personnel_read_access.contains(&patient_id)) {
            hospital_personnel_read_access.remove(&patient_id);
        };
        hospital_personnel_read_access.insert(patient_id, read_access);

        let hospital_personnel_update_access = hospital_personnel_access.borrow_mut_update();
        if (hospital_personnel_update_access.contains(&patient_id)) {
            hospital_personnel_update_access.remove(&patient_id);
        };
        hospital_personnel_update_access.insert(patient_id, update_access);

        let patient_access_log_read = patient_access_log_new(
            access_data_type_read,
            hospital_personnel_access_type_read(),
            date,
            exp_dur_read,
            *hospital_metadata,
            hospital_personnel_address,
            hospital_personnel_administrative_metadata_public,
            patient_access_log.length(),
            false,
        );
        patient_access_log.push_back(patient_access_log_read);

        let patient_access_log_update = patient_access_log_new(
            access_data_type_update,
            hospital_personnel_access_type_update(),
            date,
            exp_dur_update,
            *hospital_metadata,
            hospital_personnel_address,
            hospital_personnel_administrative_metadata_public,
            patient_access_log.length(),
            false,
        );
        patient_access_log.push_back(patient_access_log_update);
    };
}

#[test_only]
public(package) fun create_access_test(
    address_id: &AddressId,
    clock: &Clock,
    date: String,
    hospital_id_metadata: &HospitalIdMetadata,
    hospital_personnel_address: address,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    metadata: vector<String>,
    patient_id_account: &mut PatientIdAccount,
    ctx: &TxContext,
)
{
    create_access(
        address_id,
        clock, date,
        hospital_id_metadata,
        hospital_personnel_address,
        hospital_personnel_id_account,
        metadata,
        patient_id_account,
        ctx
    );
}

/// ## Params:
/// - `metadata`: vector<Base64 encoded>
///     - length = 1 for administrative
///     - length = 2 for medical
///         - 0: read
///         - 1: update
fun create_access_administrative_personnel(
    clock: &Clock,
    metadata: vector<String>,
): (HospitalPersonnelAccessData, vector<HospitalPersonnelAccessDataType>, u64)
{
    assert!(metadata.length() == 1, EInvalidMetadataLength);

    let mut hospital_personnel_access_data_types_read = vector::empty<HospitalPersonnelAccessDataType>();
    hospital_personnel_access_data_types_read.push_back(hospital_personnel_access_data_type_administrative());

    let exp_dur = 5;
    let exp_read = clock.timestamp_ms() + (exp_dur * 60 * 1000); // 5 minutes

    let hospital_personnel_access_data_read = hospital_personnel_access_data_new(
        hospital_personnel_access_data_types_read,
        exp_read,
        *metadata.borrow(0),
        option::none(),
    );

    (hospital_personnel_access_data_read, hospital_personnel_access_data_types_read, exp_dur)
}


/// ## Params:
/// - `metadata`: vector<Base64 encoded>
///     - length = 1 for administrative
///     - length = 2 for medical
///         - 0: read
///         - 1: update
/// ## Return:
/// - 0: `read_access`,
/// - 1: `read_access_data_type`,
/// - 2: `update_access`,
/// - 3: `update_access_data_type`,
fun create_access_medical_personnel(
    clock: &Clock,
    metadata: vector<String>,
): (HospitalPersonnelAccessData, vector<HospitalPersonnelAccessDataType>,
    HospitalPersonnelAccessData, vector<HospitalPersonnelAccessDataType>, u64, u64)
{
    assert!(metadata.length() == 2, EInvalidMetadataLength);

    let mut hospital_personnel_access_data_types_read = vector::empty<HospitalPersonnelAccessDataType>();
    hospital_personnel_access_data_types_read.push_back(hospital_personnel_access_data_type_medical());
    hospital_personnel_access_data_types_read.push_back(hospital_personnel_access_data_type_administrative());
    let mut hospital_personnel_access_data_types_update = vector::empty<HospitalPersonnelAccessDataType>();
    hospital_personnel_access_data_types_update.push_back(hospital_personnel_access_data_type_medical());

    let exp_dur_read = 15;
    let exp_read = clock.timestamp_ms() + (exp_dur_read * 60 * 1000); // 15 minutes
    let exp_dur_update = 2 * 60;
    let exp_update = clock.timestamp_ms() + (exp_dur_update * 60 * 1000); // 2 hours

    let hospital_personnel_access_data_read = hospital_personnel_access_data_new(
        hospital_personnel_access_data_types_read,
        exp_read,
        *metadata.borrow(0),
        option::none(),
    );
    let hospital_personnel_access_data_update = hospital_personnel_access_data_new(
        hospital_personnel_access_data_types_update,
        exp_update,
        *metadata.borrow(1),
        option::none(),
    );

    (hospital_personnel_access_data_read, hospital_personnel_access_data_types_read,
     hospital_personnel_access_data_update, hospital_personnel_access_data_types_update, exp_dur_read, exp_dur_update)
}

entry fun is_account_registered(
    address_id: &AddressId,
    ctx: &TxContext,
)
{
    let address_id_table = address_id.borrow_table();
    assert!(address_id_table.contains(ctx.sender()), EAddressNotFound);
}

/// ## Params
/// - `patient_id`: argon_hash(raw_nik)
/// - `private_metadata`: Base64 encoded
entry fun signup(
    address_id: &mut AddressId,
    patient_id: String,
    patient_id_account: &mut PatientIdAccount,
    private_metadata: String,
    ctx: &mut TxContext,
)
{
    let address_id_table = address_id.borrow_mut_table();
    assert!(!address_id_table.contains(ctx.sender()), EAddressAlreadyRegistered);

    let patient_id = encode_patient_id(patient_id);

    address_id_table.add(ctx.sender(), patient_id);

    let patient_id_account_table = patient_id_account.borrow_mut_table();

    assert!(!patient_id_account_table.contains(patient_id), EAccountAlreadyRegistered);

    let access_log = table_vec::empty<PatientAccessLog>(ctx);
    let administrative_metadata = patient_administrative_metadata_new(private_metadata);
    let medical_metadata = table_vec::empty<PatientMedicalMetadata>(ctx);

    let patient_account = patient_account_new(access_log, ctx.sender(), administrative_metadata, false, medical_metadata);
    patient_id_account_table.add(patient_id, patient_account);
}

#[test_only]
public(package) fun signup_test(
    address_id: &mut AddressId,
    patient_id: String,
    patient_id_account: &mut PatientIdAccount,
    private_metadata: String,
    ctx: &mut TxContext,
)
{
    signup(
        address_id,
        patient_id,
        patient_id_account,
        private_metadata,
        ctx
    );
}

entry fun get_account_info(
    address_id: &AddressId,
    patient_id_account: &PatientIdAccount,
    ctx: &TxContext,
): PatientAdministrativeMetadata
{
    let address_id_table = address_id.borrow_table();
    let patient_id = *address_id_table.borrow(ctx.sender());
    let patient_id_account_table = patient_id_account.borrow_table();
    let patient_account = patient_id_account_table.borrow(patient_id);

    *patient_account.borrow_administrative_metadata()
}

/// ## Params
/// - `patient_id`: argon_hash(raw_id)
///
/// ## Return:
/// 0: state_code
///     - 0 means need auth
///     - 1 means need profile completion
///     - 2 means no action
entry fun get_account_state(
    patient_id: String,
    patient_id_account: &PatientIdAccount,
): u64
{
    let patient_id = encode_patient_id(patient_id);
    let patient_id_account_table = patient_id_account.borrow_table();

    if (!patient_id_account_table.contains(patient_id)) {
        return 0
    };

    let patient_account = patient_id_account_table.borrow(patient_id);

    if (!patient_account.borrow_is_profile_completed()) {
        return 1
    };

    2
}

/// ## Return:
/// 0: public administrative data
/// 1: hospital name
entry fun get_hospital_personnel_info(
    address_id: &AddressId,
    hospital_id_metadata: &HospitalIdMetadata,
    hospital_personnel_address: address,
    hospital_personnel_id_account: &HospitalPersonnelIdAccount,
    ctx: &TxContext,
): (String, String)
{
    let address_id_table = address_id.borrow_table();

    assert!(address_id_table.contains(ctx.sender()), EAccountNotFound);

    let hospital_personnel_id = *address_id_table.borrow(hospital_personnel_address);
    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_table();
    let hospital_personnel_account = hospital_personnel_id_account_table.borrow(hospital_personnel_id);
    let hospital_personnel_administrative_metadata = hospital_personnel_account.borrow_administrative_metadata().borrow();

    let hospital_id_metadata_table = hospital_id_metadata.borrow_table();
    let hospital_id_metadata_vec = hospital_id_metadata.borrow_vec();
    let hospital_metadata_index = hospital_id_metadata_table.borrow(*hospital_personnel_account.borrow_hospital_id());
    let hospital_metadata = hospital_id_metadata_vec.borrow(*hospital_metadata_index).borrow_hospital_metadata();

    let public_data = *hospital_personnel_administrative_metadata.borrow_public_metadata();
    let hospital_name = *hospital_metadata.borrow_name();

    (public_data, hospital_name)
}

entry fun get_access_log(
    address_id: &AddressId,
    cursor: u64,
    patient_id_account: &PatientIdAccount,
    size: u64,
    ctx: &TxContext,
): vector<PatientAccessLog>
{
    let address_id_table = address_id.borrow_table();
    let patient_id = *address_id_table.borrow(ctx.sender());
    let patient_id_account_table = patient_id_account.borrow_table();
    let patient_account = patient_id_account_table.borrow(patient_id);
    let patient_access_log = patient_account.borrow_access_log();

    let patient_access_log_length = patient_access_log.length();

    let mut result = vector::empty<PatientAccessLog>();

    if (cursor >= patient_access_log_length) {
        return result
    };

    let size = std::u64::min(size, 10);
    let end_idx = patient_access_log_length - cursor - 1;
    let mut start_idx = end_idx + 1 - std::u64::min(size, end_idx + 1);
    let mut curr_idx = end_idx;

    while (start_idx <= end_idx) {
        result.push_back(*patient_access_log.borrow(curr_idx));
        start_idx = start_idx + 1;

        if (curr_idx > 0) {
            curr_idx = curr_idx - 1;
        };
    };

    result
}

entry fun get_medical_record(
    address_id: &AddressId,
    index: u64,
    patient_id_account: &PatientIdAccount,
    ctx: &TxContext,
): PatientMedicalMetadata
{
    let address_id_table = address_id.borrow_table();
    let patient_id = *address_id_table.borrow(ctx.sender());
    let patient_id_account_table = patient_id_account.borrow_table();
    let patient_account = patient_id_account_table.borrow(patient_id);
    let patient_medical_metadata = patient_account.borrow_medical_metadata();

    *patient_medical_metadata.borrow(index)
}

entry fun get_medical_records(
    address_id: &AddressId,
    cursor: u64,
    patient_id_account: &PatientIdAccount,
    size: u64,
    ctx: &TxContext,
): vector<PatientMedicalMetadata>
{
    let address_id_table = address_id.borrow_table();
    let patient_id = *address_id_table.borrow(ctx.sender());
    let patient_id_account_table = patient_id_account.borrow_table();
    let patient_account = patient_id_account_table.borrow(patient_id);
    let patient_medical_metadata = patient_account.borrow_medical_metadata();

    let patient_medical_metadata_length = patient_medical_metadata.length();

    let mut result = vector::empty<PatientMedicalMetadata>();

    if (cursor >= patient_medical_metadata_length) {
        return result
    };

    let size = std::u64::min(size, 10);
    let end_idx = patient_medical_metadata_length - cursor - 1;
    let mut start_idx = end_idx + 1 - std::u64::min(size, end_idx + 1);
    let mut curr_idx = end_idx;

    while (start_idx <= end_idx) {
        result.push_back(*patient_medical_metadata.borrow(curr_idx));
        start_idx = start_idx + 1;

        if (curr_idx > 0) {
            curr_idx = curr_idx - 1;
        };
    };

    result
}

entry fun revoke_access(
    address_id: &AddressId,
    hospital_personnel_address: address,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    index: u64,
    patient_id_account: &mut PatientIdAccount,
    ctx: &TxContext,
)
{
    let address_id_table = address_id.borrow_table();
    let patient_id = *address_id_table.borrow(ctx.sender());
    let hospital_personnel_id = *address_id_table.borrow(hospital_personnel_address);

    let patient_id_account_table = patient_id_account.borrow_mut_table();

    assert!(patient_id_account_table.contains(patient_id), EAccountNotFound);

    let patient_account = patient_id_account_table.borrow_mut(patient_id);
    let patient_access_logs = patient_account.borrow_mut_access_log();
    let patient_access_log = patient_access_logs.borrow_mut(index);
    patient_access_log.set_is_revoked(true);

    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_mut_table();
    let hospital_personnel_account = hospital_personnel_id_account_table.borrow_mut(hospital_personnel_id);
    let hospital_personnel_access = hospital_personnel_account.borrow_mut_access();

    let hospital_personnel_read_access = hospital_personnel_access.borrow_mut().borrow_mut_read();
    if (hospital_personnel_read_access.contains(&patient_id) && patient_access_log.borrow_access_type() == hospital_personnel_access_type_read()) {
        hospital_personnel_read_access.remove(&patient_id);
    };
    let hospital_personnel_update_access = hospital_personnel_access.borrow_mut().borrow_mut_update();
    if (hospital_personnel_update_access.contains(&patient_id) && patient_access_log.borrow_access_type() == hospital_personnel_access_type_update()) {
        hospital_personnel_update_access.remove(&patient_id);
    };
}

/// ## Params
/// - `private_metadata`: Base64 encoded
entry fun update_administrative_metadata(
    address_id: &AddressId,
    patient_id_account: &mut PatientIdAccount,
    private_metadata: String,
    ctx: &TxContext
)
{
    let address_id_table = address_id.borrow_table();
    let patient_id = *address_id_table.borrow(ctx.sender());
    let patient_id_account_table = patient_id_account.borrow_mut_table();
    let patient_account = patient_id_account_table.borrow_mut(patient_id);

    let patient_administrative_metadata = patient_account.borrow_mut_administrative_metadata();
    patient_administrative_metadata.set_private_metadata(private_metadata);
    patient_account.set_is_profile_completed(true);
}
