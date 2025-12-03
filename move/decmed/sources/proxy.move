module decmed::proxy;

use decmed::std_enum_hospital_personnel_access_data_type::{
    administrative as hospital_personnel_access_data_type_administrative,
    medical as hospital_personnel_access_data_type_medical,
};
use decmed::std_enum_hospital_personnel_role::{
    medical_personnel as hospital_personnel_role_medical_personnel,
};
use decmed::std_enum_hospital_personnel_role::{
    HospitalPersonnelRole,
};
use decmed::shared::{
    GlobalAdminCap,
    ProxyCap,

    transfer_proxy_cap,
};
use decmed::std_struct_address_id::AddressId;
use decmed::std_struct_hospital_personnel_id_account::HospitalPersonnelIdAccount;
use decmed::std_struct_patient_administrative_metadata::PatientAdministrativeMetadata;
use decmed::std_struct_patient_id_account::PatientIdAccount;
use decmed::std_struct_patient_medical_metadata::{
    PatientMedicalMetadata,

    new as patient_medical_metadata_new,
};

use iota::clock::Clock;

use std::string::String;

const EAccessExpired: u64 = 4000;
const EAccessNotFound: u64 = 4001;
const EAccountNotFound: u64 = 4002;
const EAddressNotFound: u64 = 4003;
const EInvalidAccessType: u64 = 4004;
const EMedicalRecordCreationLimit: u64 = 4005;
const EMedicalRecordNotFound: u64 = 4006;

entry fun create_capability(
    proxy_address: address,
    _: &GlobalAdminCap,
    ctx: &mut TxContext,
)
{
   transfer_proxy_cap(proxy_address, ctx);
}

entry fun create_medical_record(
    address_id: &AddressId,
    clock: &Clock,
    hospital_personnel_address: address,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    metadata: String,
    patient_address: address,
    patient_id_account: &mut PatientIdAccount,
    _: &ProxyCap,
)
{
    let address_id_table = address_id.borrow_table();

    let patient_id = address_id_table.borrow(patient_address);
    let hospital_personnel_id = address_id_table.borrow(hospital_personnel_address);

    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_mut_table();
    let hospital_personnel_account = hospital_personnel_id_account_table.borrow_mut(*hospital_personnel_id);
    let hospital_personnel_access = hospital_personnel_account.borrow_mut_access().borrow_mut();
    let hospital_personnel_update_access = hospital_personnel_access.borrow_mut_update();

    assert!(hospital_personnel_update_access.contains(patient_id), EAccessNotFound);
    let update_access = hospital_personnel_update_access.get(patient_id);

    assert!(update_access.borrow_medical_metadata_index().is_none(), EMedicalRecordCreationLimit);

    if (update_access.borrow_exp() < clock.timestamp_ms()) {
        hospital_personnel_update_access.remove(patient_id);
        assert!(false, EAccessExpired);
    };

    let patient_id_account_table = patient_id_account.borrow_mut_table();
    let patient_account = patient_id_account_table.borrow_mut(*patient_id);
    let patient_medical_metadata = patient_account.borrow_mut_medical_metadata();

    let index = patient_medical_metadata.length();
    let medical_metadata = patient_medical_metadata_new(
        index,
        metadata,
    );

    patient_medical_metadata.push_back(medical_metadata);

    let update_access = hospital_personnel_update_access.get_mut(patient_id);
    update_access.set_medical_metadata_index(option::some(index));
}

#[test_only]
public(package) fun create_medical_record_test(
    address_id: &AddressId,
    clock: &Clock,
    hospital_personnel_address: address,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    metadata: String,
    patient_address: address,
    patient_id_account: &mut PatientIdAccount,
    proxy_cap: &ProxyCap,
)
{
    create_medical_record(
        address_id,
        clock,
        hospital_personnel_address,
        hospital_personnel_id_account,
        metadata,
        patient_address,
        patient_id_account,
        proxy_cap,
    );
}

entry fun get_administrative_data(
    address_id: &AddressId,
    clock: &Clock,
    hospital_personnel_address: address,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    patient_address: address,
    patient_id_account: &PatientIdAccount,
    _: &ProxyCap,
): PatientAdministrativeMetadata
{
    let address_id_table = address_id.borrow_table();
    let hospital_personnel_id = *address_id_table.borrow(hospital_personnel_address);
    let patient_id = *address_id_table.borrow(patient_address);

    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_mut_table();
    let hospital_personnel_account = hospital_personnel_id_account_table.borrow_mut(hospital_personnel_id);
    let hospital_personnel_role = hospital_personnel_account.borrow_role();

    let patient_id_account_table = patient_id_account.borrow_table();
    let patient_account = patient_id_account_table.borrow(patient_id);

    if (hospital_personnel_role == hospital_personnel_role_medical_personnel()) {
        // Check access
        let hospital_personnel_access = hospital_personnel_account.borrow_mut_access().borrow_mut();
        let hospital_personnel_update_access = hospital_personnel_access.borrow_mut_update();

        assert!(hospital_personnel_update_access.contains(&patient_id), EAccessNotFound);
        let update_access = hospital_personnel_update_access.get(&patient_id);

        if (update_access.borrow_exp() < clock.timestamp_ms()) {
            hospital_personnel_update_access.remove(&patient_id);
            assert!(false, EAccessExpired);
        };
    } else {
        // Check access
        let hospital_personnel_access = hospital_personnel_account.borrow_mut_access().borrow_mut();
        let hospital_personnel_read_access = hospital_personnel_access.borrow_mut_read();

        assert!(hospital_personnel_read_access.contains(&patient_id), EAccessNotFound);
        let read_access = hospital_personnel_read_access.get(&patient_id);

        let read_access_types = read_access.borrow_access_data_types();
        assert!(read_access_types.contains(&hospital_personnel_access_data_type_administrative()), EInvalidAccessType);

        if (read_access.borrow_exp() < clock.timestamp_ms()) {
            hospital_personnel_read_access.remove(&patient_id);
            assert!(false, EAccessExpired);
        };
    };

    let patient_administrative_metadata = patient_account.borrow_administrative_metadata();

    *patient_administrative_metadata
}

#[test_only]
public(package) fun get_administrative_data_test(
    address_id: &AddressId,
    clock: &Clock,
    hospital_personnel_address: address,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    patient_address: address,
    patient_id_account: &PatientIdAccount,
    proxy_cap: &ProxyCap,
): PatientAdministrativeMetadata
{
    get_administrative_data(
        address_id,
        clock,
        hospital_personnel_address,
        hospital_personnel_id_account,
        patient_address,
        patient_id_account,
        proxy_cap
    )
}

entry fun get_hospital_personnel_role(
    address_id: &AddressId,
    hospital_personnel_id_account: &HospitalPersonnelIdAccount,
    hospital_personnel_address: address,
    _: &ProxyCap,
): HospitalPersonnelRole
{
    let address_id_table = address_id.borrow_table();
    let hospital_personnel_id = *address_id_table.borrow(hospital_personnel_address);
    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_table();

    let hospital_personnel_account = hospital_personnel_id_account_table.borrow(hospital_personnel_id);
    let role = *hospital_personnel_account.borrow_role();

    role
}

/// ## Returns:
/// 1: prev_index
/// 2: next_index
entry fun get_medical_record(
    address_id: &AddressId,
    clock: &Clock,
    hospital_personnel_address: address,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    index: u64,
    patient_address: address,
    patient_id_account: &PatientIdAccount,
    _: &ProxyCap,
): (PatientMedicalMetadata, PatientAdministrativeMetadata, u64, Option<u64>, Option<u64>)
{
    let address_id_table = address_id.borrow_table();
    let hospital_personnel_id = *address_id_table.borrow(hospital_personnel_address);
    let patient_id = *address_id_table.borrow(patient_address);

    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_mut_table();
    let hospital_personnel_account = hospital_personnel_id_account_table.borrow_mut(hospital_personnel_id);

    let patient_id_account_table = patient_id_account.borrow_table();
    let patient_account = patient_id_account_table.borrow(patient_id);

    // Check access
    let hospital_personnel_access = hospital_personnel_account.borrow_mut_access().borrow_mut();
    let hospital_personnel_read_access = hospital_personnel_access.borrow_mut_read();

    assert!(hospital_personnel_read_access.contains(&patient_id), EAccessNotFound);
    let read_access = hospital_personnel_read_access.get(&patient_id);

    let read_access_types = read_access.borrow_access_data_types();
    assert!(read_access_types.contains(&hospital_personnel_access_data_type_medical()), EInvalidAccessType);

    if (read_access.borrow_exp() < clock.timestamp_ms()) {
        hospital_personnel_read_access.remove(&patient_id);
        assert!(false, EAccessExpired);
    };

    let patient_medical_metadata = patient_account.borrow_medical_metadata();
    let medical_metadata = patient_medical_metadata.borrow(patient_medical_metadata.length() - index - 1);

    let mut next_index = option::some(index + 1);
    let mut prev_index = option::none<u64>();

    if (patient_medical_metadata.length() == index + 1) {
        next_index = option::none()
    };
    if (index > 0) {
        prev_index = option::some(index - 1)
    };

    let patient_administrative_metadata = patient_account.borrow_administrative_metadata();

    (*medical_metadata, *patient_administrative_metadata, patient_medical_metadata.length() - index - 1, prev_index, next_index)
}

#[test_only]
public(package) fun get_medical_record_test(
    address_id: &AddressId,
    clock: &Clock,
    hospital_personnel_address: address,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    index: u64,
    patient_address: address,
    patient_id_account: &PatientIdAccount,
    proxy_cap: &ProxyCap,
): (PatientMedicalMetadata, PatientAdministrativeMetadata, u64, Option<u64>, Option<u64>)
{
    get_medical_record(
        address_id,
        clock,
        hospital_personnel_address,
        hospital_personnel_id_account,
        index,
        patient_address,
        patient_id_account,
        proxy_cap
    )
}

entry fun get_medical_record_update(
    address_id: &AddressId,
    clock: &Clock,
    hospital_personnel_address: address,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    index: u64,
    patient_address: address,
    patient_id_account: &PatientIdAccount,
    _: &ProxyCap,
): (PatientMedicalMetadata, PatientAdministrativeMetadata)
{
    let address_id_table = address_id.borrow_table();
    let hospital_personnel_id = *address_id_table.borrow(hospital_personnel_address);
    let patient_id = *address_id_table.borrow(patient_address);

    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_mut_table();
    let hospital_personnel_account = hospital_personnel_id_account_table.borrow_mut(hospital_personnel_id);

    let patient_id_account_table = patient_id_account.borrow_table();
    let patient_account = patient_id_account_table.borrow(patient_id);

    // Check access
    let hospital_personnel_access = hospital_personnel_account.borrow_mut_access().borrow_mut();
    let hospital_personnel_update_access = hospital_personnel_access.borrow_mut_update();
    let update_access = hospital_personnel_update_access.get(&patient_id);
    let update_access_types = update_access.borrow_access_data_types();

    assert!(update_access_types.contains(&hospital_personnel_access_data_type_medical()), EInvalidAccessType);
    assert!(update_access.borrow_medical_metadata_index().is_some(), EMedicalRecordNotFound);
    assert!(update_access.borrow_medical_metadata_index().borrow() == index, EMedicalRecordNotFound);

    if (update_access.borrow_exp() < clock.timestamp_ms()) {
        hospital_personnel_update_access.remove(&patient_id);
        assert!(false, EAccessExpired);
    };

    let patient_medical_metadata = patient_account.borrow_medical_metadata();
    let medical_metadata = patient_medical_metadata.borrow(index);

    let administrative_metadata = patient_account.borrow_administrative_metadata();

    (*medical_metadata, *administrative_metadata)
}

entry fun is_patient_registered(
    address_id: &AddressId,
    patient_id_account: &PatientIdAccount,
    patient_address: address,
    _: &ProxyCap,
)
{
    let address_id_table = address_id.borrow_table();

    assert!(address_id_table.contains(patient_address), EAddressNotFound);

    let patient_id = *address_id_table.borrow(patient_address);
    let patient_id_account_table = patient_id_account.borrow_table();

    assert!(patient_id_account_table.contains(patient_id), EAccountNotFound);
}

entry fun update_medical_record(
    address_id: &AddressId,
    clock: &Clock,
    hospital_personnel_address: address,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    metadata: String,
    patient_address: address,
    patient_id_account: &mut PatientIdAccount,
    _: &ProxyCap,
)
{
    let address_id_table = address_id.borrow_table();

    let patient_id = address_id_table.borrow(patient_address);
    let hospital_personnel_id = address_id_table.borrow(hospital_personnel_address);

    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_mut_table();
    let hospital_personnel_account = hospital_personnel_id_account_table.borrow_mut(*hospital_personnel_id);
    let hospital_personnel_access = hospital_personnel_account.borrow_mut_access().borrow_mut();
    let hospital_personnel_update_access = hospital_personnel_access.borrow_mut_update();

    assert!(hospital_personnel_update_access.contains(patient_id), EAccessNotFound);
    let update_access = hospital_personnel_update_access.get(patient_id);
    let update_access_types = update_access.borrow_access_data_types();

    assert!(update_access_types.contains(&hospital_personnel_access_data_type_medical()), EInvalidAccessType);
    assert!(update_access.borrow_medical_metadata_index().is_some(), EMedicalRecordNotFound);

    let index = *update_access.borrow_medical_metadata_index().borrow();

    if (update_access.borrow_exp() < clock.timestamp_ms()) {
        hospital_personnel_update_access.remove(patient_id);
        assert!(false, EAccessExpired);
    };

    let patient_id_account_table = patient_id_account.borrow_mut_table();
    let patient_account = patient_id_account_table.borrow_mut(*patient_id);
    let patient_medical_metadata = patient_account.borrow_mut_medical_metadata();

    let medical_metadata = patient_medical_metadata_new(
        index,
        metadata,
    );

    patient_medical_metadata.push_back(medical_metadata);
    patient_medical_metadata.swap_remove(index);
}

#[test_only]
public(package) fun update_medical_record_test(
    address_id: &AddressId,
    clock: &Clock,
    hospital_personnel_address: address,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    metadata: String,
    patient_address: address,
    patient_id_account: &mut PatientIdAccount,
    proxy_cap: &ProxyCap,
)
{
    update_medical_record(
        address_id,
        clock,
        hospital_personnel_address,
        hospital_personnel_id_account,
        metadata,
        patient_address,
        patient_id_account,
        proxy_cap
    );
}
