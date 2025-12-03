module decmed::shared_tests;
use decmed::admin::{
    create_activation_key_test as create_activation_key_test_by_global_admin
};

use decmed::hospital_personnel::{
    use_activation_key_test,
    signup_test as signup_test_by_hospital_personnel,
    create_activation_key_test as create_activation_key_test_by_hospital_admin
};

use decmed::patient::{
    signup_test as signup_test_by_patient,
};

use decmed::shared::{
    transfer_proxy_cap,
    transfer_global_admin_cap,
    GlobalAdminCap,
};

use decmed::std_enum_hospital_personnel_role::{
    HospitalPersonnelRole,
    admin as hospital_personnel_role_admin,
    administrative_personnel as hospital_personnel_role_administrative_personnel,
    medical_personnel as hospital_personnel_role_medical_personnel,
};

use decmed::std_struct_address_id::{
    AddressId,
    default as address_id_default,
};
use decmed::std_struct_hospital_id_metadata::{
    HospitalIdMetadata,
    default as hospital_id_metadata_default,
};
use decmed::std_struct_hospital_personnel_id_account::{
    HospitalPersonnelIdAccount,
    default as hospital_personnel_id_account_default,
};
use decmed::std_struct_patient_id_account::{
    PatientIdAccount,
    default as patient_id_account_default,
};

use std::string::{Self, String};

use iota::test_scenario;

#[test_only]
const PUBLISHER_ADDR: address = @0xA;
#[test_only]
const PROXY_ADDR: address = @0xAAAA;
#[test_only]
const PATIENT_ADDR: address = @0xAA;
#[test_only]
const PATIENT_2_ADDR: address = @0xAA2;
#[test_only]
const HOSPITAL_ADMIN_ADDR: address = @0x0A;
#[test_only]
const ADMINISTRATIVE_PERSONNEL_ADDR: address = @0xBB;
#[test_only]
const MEDICAL_PERSONNEL_ADDR: address = @0xCC;

#[test_only]
public(package) fun patient_id(index: u64): String
{
    let mut id = string::utf8(b"62596e08192bf1b7787de085eda79ba7");
    id.append(index.to_string());
    id
}

#[test_only]
public(package) fun patient_medical_metadata(index: u64): String
{
    let mut id = string::utf8(b"UGF0aWVudE1lZGljYWxNZXRhZGF0YQ==");
    id.append(index.to_string());
    id
}

#[test_only]
public(package) fun patient_private_administrative_metadata(): String
{
    string::utf8(b"UGF0aWVudFByaXZhdGVBZG1pbmlzdHJhdGl2ZU1ldGFkYXRh")
}


#[test_only]
public(package) fun hospital_personnel_id(role: HospitalPersonnelRole): String
{
    if (role == hospital_personnel_role_admin()) {
        return string::utf8(b"7f7b18da3390c5b78ca58a2950ff9932")
    };

    if (role == hospital_personnel_role_administrative_personnel()) {
        return string::utf8(b"5c3b22c5acd5e8c2b7be9e2e1de96ceb")
    };

    string::utf8(b"00b6b71874960b441bf8566fee9e0672")
}

#[test_only]
public(package) fun hospital_personnel_activation_key(role: HospitalPersonnelRole): String
{
    if (role == hospital_personnel_role_admin()) {
        return string::utf8(b"QWN0aXZhdGlvbktleUFkbWlu")
    };

    if (role == hospital_personnel_role_administrative_personnel()) {
        return string::utf8(b"QWN0aXZhdGlvbktleUFkbWluaXN0cmF0aXZl")
    };

    string::utf8(b"QWN0aXZhdGlvbktleU1lZGljYWw=")
}

#[test_only]
public(package) fun hospital_personnel_role_bytes(role: HospitalPersonnelRole): vector<u8>
{
    if (role == hospital_personnel_role_admin()) {
        return b"Admin"
    };

    if (role == hospital_personnel_role_administrative_personnel()) {
        return b"AdministrativePersonnel"
    };

    b"MedicalPersonnel"
}

#[test_only]
public(package) fun account_metadata(): String
{
    string::utf8(b"aG9zcGl0YWxhZG1pbm1ldGFkYXRh")
}

#[test_only]
public(package) fun hospital_id(): String
{
    string::utf8(b"1a70c24b6181a74dababba9dab04ec8a")
}

#[test_only]
public(package) fun hospital_name(): String
{
    string::utf8(b"HospitalName")
}

#[test_only]
public(package) fun hospital_personnel_private_administrative_metadata(role: HospitalPersonnelRole): String
{
    if (role == hospital_personnel_role_admin()) {
        return string::utf8(b"UHJpdmF0ZUFkbUFkbWlu")
    };

    if (role == hospital_personnel_role_administrative_personnel()) {
        return string::utf8(b"UHJpdmF0ZUFkbUFkbWluaXN0cmF0aXZl")
    };

    string::utf8(b"UHJpdmF0ZUFkbU1lZGljYWw=")
}

#[test_only]
public(package) fun hospital_personnel_public_administrative_metadata(role: HospitalPersonnelRole): String
{
    if (role == hospital_personnel_role_admin()) {
        return string::utf8(b"UHVibGljQWRtQWRtaW4=")
    };

    if (role == hospital_personnel_role_administrative_personnel()) {
        return string::utf8(b"UHVibGljQWRtQWRtaW5pc3RyYXRpdmU=")
    };

    string::utf8(b"UHVibGljQWRtTWVkaWNhbA==")
}


#[test_only]
public(package) fun setup_shared_objects(ctx: &mut TxContext)
{
    let address_id = address_id_default(ctx);
    let hospital_personnel_id_account = hospital_personnel_id_account_default(ctx);
    let patient_id_account = patient_id_account_default(ctx);
    let hospital_id_metadata = hospital_id_metadata_default(ctx);

    transfer_global_admin_cap(PUBLISHER_ADDR, ctx);
    transfer_proxy_cap(PROXY_ADDR, ctx);
    transfer::public_share_object(address_id);
    transfer::public_share_object(hospital_personnel_id_account);
    transfer::public_share_object(patient_id_account);
    transfer::public_share_object(hospital_id_metadata);
}

#[test_only]
public(package) fun setup_data(
    scenario: &mut test_scenario::Scenario,
)
{
    let mut address_id = test_scenario::take_shared<AddressId>(scenario);
    let mut hospital_personnel_id_account = test_scenario::take_shared<HospitalPersonnelIdAccount>(scenario);
    let mut patient_id_account = test_scenario::take_shared<PatientIdAccount>(scenario);
    let mut hospital_id_metadata = test_scenario::take_shared<HospitalIdMetadata>(scenario);
    let global_admin_cap = test_scenario::take_from_address<GlobalAdminCap>(scenario, PUBLISHER_ADDR);

    test_scenario::next_tx(scenario, PUBLISHER_ADDR);

    {
        create_activation_key_test_by_global_admin(
            hospital_personnel_activation_key(hospital_personnel_role_admin()),
            hospital_personnel_id(hospital_personnel_role_admin()),
            account_metadata(),
            hospital_id(),
            &mut hospital_id_metadata,
            hospital_name(),
            &mut hospital_personnel_id_account,
            &global_admin_cap
        );
    };


    test_scenario::next_tx(scenario, HOSPITAL_ADMIN_ADDR);

    {
        use_activation_key_test(
            hospital_personnel_activation_key(hospital_personnel_role_admin()),
            hospital_id(),
            &mut hospital_personnel_id_account,
            hospital_personnel_id(hospital_personnel_role_admin()),
        );

        signup_test_by_hospital_personnel(
            hospital_personnel_activation_key(hospital_personnel_role_admin()),
            &mut address_id,
            hospital_id(),
            &mut hospital_personnel_id_account,
            hospital_personnel_id(hospital_personnel_role_admin()),
            hospital_personnel_private_administrative_metadata(hospital_personnel_role_admin()),
            hospital_personnel_public_administrative_metadata(hospital_personnel_role_admin()),
            test_scenario::ctx(scenario),
        );

        create_activation_key_test_by_hospital_admin(
            &address_id,
            hospital_personnel_activation_key(hospital_personnel_role_admin()),
            &mut hospital_personnel_id_account,
            account_metadata(),
            hospital_personnel_activation_key(hospital_personnel_role_administrative_personnel()),
            hospital_personnel_id(hospital_personnel_role_administrative_personnel()),
            hospital_personnel_role_bytes(hospital_personnel_role_administrative_personnel()),
            test_scenario::ctx(scenario),
        );

        create_activation_key_test_by_hospital_admin(
            &address_id,
            hospital_personnel_activation_key(hospital_personnel_role_admin()),
            &mut hospital_personnel_id_account,
            account_metadata(),
            hospital_personnel_activation_key(hospital_personnel_role_medical_personnel()),
            hospital_personnel_id(hospital_personnel_role_medical_personnel()),
            hospital_personnel_role_bytes(hospital_personnel_role_medical_personnel()),
            test_scenario::ctx(scenario),
        );
    };

    test_scenario::next_tx(scenario, ADMINISTRATIVE_PERSONNEL_ADDR);

    {
        use_activation_key_test(
            hospital_personnel_activation_key(hospital_personnel_role_administrative_personnel()),
            hospital_id(),
            &mut hospital_personnel_id_account,
            hospital_personnel_id(hospital_personnel_role_administrative_personnel()),
        );

        signup_test_by_hospital_personnel(
            hospital_personnel_activation_key(hospital_personnel_role_administrative_personnel()),
            &mut address_id,
            hospital_id(),
            &mut hospital_personnel_id_account,
            hospital_personnel_id(hospital_personnel_role_administrative_personnel()),
            hospital_personnel_private_administrative_metadata(hospital_personnel_role_administrative_personnel()),
            hospital_personnel_public_administrative_metadata(hospital_personnel_role_administrative_personnel()),
            test_scenario::ctx(scenario),
        );
    };

    test_scenario::next_tx(scenario, MEDICAL_PERSONNEL_ADDR);

    {
        use_activation_key_test(
            hospital_personnel_activation_key(hospital_personnel_role_medical_personnel()),
            hospital_id(),
            &mut hospital_personnel_id_account,
            hospital_personnel_id(hospital_personnel_role_medical_personnel()),
        );

        signup_test_by_hospital_personnel(
            hospital_personnel_activation_key(hospital_personnel_role_medical_personnel()),
            &mut address_id,
            hospital_id(),
            &mut hospital_personnel_id_account,
            hospital_personnel_id(hospital_personnel_role_medical_personnel()),
            hospital_personnel_private_administrative_metadata(hospital_personnel_role_medical_personnel()),
            hospital_personnel_public_administrative_metadata(hospital_personnel_role_medical_personnel()),
            test_scenario::ctx(scenario),
        );
    };

    test_scenario::next_tx(scenario, PATIENT_ADDR);

    {
        signup_test_by_patient(
            &mut address_id,
            patient_id(1),
            &mut patient_id_account,
            patient_private_administrative_metadata(),
            test_scenario::ctx(scenario),
        );
    };

    test_scenario::next_tx(scenario, PATIENT_2_ADDR);

    {
        signup_test_by_patient(
            &mut address_id,
            patient_id(2),
            &mut patient_id_account,
            patient_private_administrative_metadata(),
            test_scenario::ctx(scenario),
        );
    };

    test_scenario::return_shared(address_id);
    test_scenario::return_shared(hospital_personnel_id_account);
    test_scenario::return_shared(patient_id_account);
    test_scenario::return_shared(hospital_id_metadata);
    test_scenario::return_to_address(PUBLISHER_ADDR, global_admin_cap);
}
