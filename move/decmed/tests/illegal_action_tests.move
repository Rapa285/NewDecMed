#[test_only]
module decmed::decmed_illegal_action_tests;

use decmed::shared_tests::{
    patient_id,
    patient_medical_metadata,
    setup_data,
    setup_shared_objects,
};

use decmed::patient::{
    create_access_test,
};

use decmed::proxy::{
    create_medical_record_test,
    get_administrative_data_test,
    get_medical_record_test,
    update_medical_record_test,
};

use decmed::shared::{
    encode_patient_id,
    ProxyCap,
};

use decmed::std_struct_address_id::{
    AddressId,
};
use decmed::std_struct_hospital_id_metadata::{
    HospitalIdMetadata,
};
use decmed::std_struct_hospital_personnel_id_account::{
    HospitalPersonnelIdAccount,
};
use decmed::std_struct_patient_id_account::{
    PatientIdAccount,
};
use decmed::std_struct_patient_medical_metadata::{
    new as patient_medical_metadata_new,
};


use std::string::{Self, String};

use iota::clock::{Self, Clock};
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
const ADMINISTRATIVE_PERSONNEL_ADDR: address = @0xBB;
#[test_only]
const MEDICAL_PERSONNEL_ADDR: address = @0xCC;

#[test_only]
fun create_access(
    clock: &Clock,
    hospital_personnel_address: address,
    metadata: vector<String>,
    scenario: &mut test_scenario::Scenario,
)
{
    let address_id = test_scenario::take_shared<AddressId>(scenario);
    let mut hospital_personnel_id_account = test_scenario::take_shared<HospitalPersonnelIdAccount>(scenario);
    let mut patient_id_account = test_scenario::take_shared<PatientIdAccount>(scenario);
    let hospital_id_metadata = test_scenario::take_shared<HospitalIdMetadata>(scenario);

    create_access_test(
        &address_id,
        clock,
        string::utf8(b"2025-07-28T14:40:49+00:00"),
        &hospital_id_metadata,
        hospital_personnel_address,
        &mut hospital_personnel_id_account,
        metadata,
        &mut patient_id_account,
        test_scenario::ctx(scenario),
    );

    test_scenario::return_shared(address_id);
    test_scenario::return_shared(hospital_personnel_id_account);
    test_scenario::return_shared(patient_id_account);
    test_scenario::return_shared(hospital_id_metadata);
}

#[test_only]
fun add_medical_record(
    scenario: &test_scenario::Scenario,
)
{
    let address_id = test_scenario::take_shared<AddressId>(scenario);
    let hospital_personnel_id_account = test_scenario::take_shared<HospitalPersonnelIdAccount>(scenario);
    let mut patient_id_account = test_scenario::take_shared<PatientIdAccount>(scenario);
    let hospital_id_metadata = test_scenario::take_shared<HospitalIdMetadata>(scenario);

    let patient_id_account_table = patient_id_account.borrow_mut_table();
    let patient_account = patient_id_account_table.borrow_mut(encode_patient_id(patient_id(1)));
    let patient_medical_metadata = patient_account.borrow_mut_medical_metadata();
    patient_medical_metadata.push_back(patient_medical_metadata_new(0, patient_medical_metadata(1)));
    patient_medical_metadata.push_back(patient_medical_metadata_new(1, patient_medical_metadata(2)));

    test_scenario::return_shared(address_id);
    test_scenario::return_shared(hospital_personnel_id_account);
    test_scenario::return_shared(patient_id_account);
    test_scenario::return_shared(hospital_id_metadata);
}

#[test, expected_failure(abort_code = ::decmed::proxy::EInvalidAccessType)]
// Read access medical part of medical record
// by administrative personnel
fun test_ill_01()
{
    let mut scenario_val = test_scenario::begin(PUBLISHER_ADDR);
    let scenario = &mut scenario_val;

    let mut clck = clock::create_for_testing(test_scenario::ctx(scenario));

    setup_shared_objects(test_scenario::ctx(scenario));
    test_scenario::next_tx(scenario, PUBLISHER_ADDR);
    setup_data(scenario);
    test_scenario::next_tx(scenario, PUBLISHER_ADDR);
    add_medical_record(scenario);

    test_scenario::next_tx(scenario, PATIENT_ADDR);

    {
        let mut metadata = vector::empty<String>();
        metadata.push_back(string::utf8(b"ReadAccessForAdmPersonnel"));

        create_access(
            &clck,
            ADMINISTRATIVE_PERSONNEL_ADDR,
            metadata,
            scenario
        );
    };

    test_scenario::next_tx(scenario, PROXY_ADDR);

    {
        let address_id = test_scenario::take_shared<AddressId>(scenario);
        let mut hospital_personnel_id_account = test_scenario::take_shared<HospitalPersonnelIdAccount>(scenario);
        let patient_id_account = test_scenario::take_shared<PatientIdAccount>(scenario);
        let hospital_id_metadata = test_scenario::take_shared<HospitalIdMetadata>(scenario);
        let proxy_cap = test_scenario::take_from_address<ProxyCap>(scenario, PROXY_ADDR);
        clck.set_for_testing(3 * 60 * 1000);

        let (_, _, _, _, _) = get_medical_record_test(
            &address_id,
            &clck,
            ADMINISTRATIVE_PERSONNEL_ADDR,
            &mut hospital_personnel_id_account,
            0,
            PATIENT_ADDR,
            &patient_id_account,
            &proxy_cap
        );

        test_scenario::return_shared(address_id);
        test_scenario::return_shared(hospital_personnel_id_account);
        test_scenario::return_shared(patient_id_account);
        test_scenario::return_shared(hospital_id_metadata);
        test_scenario::return_to_address(PROXY_ADDR, proxy_cap);
    };

    clck.destroy_for_testing();
    test_scenario::end(scenario_val);
}

#[test, expected_failure(abort_code = ::decmed::proxy::EAccessExpired)]
// Read access administrataive part of medical record
// by administrative personnel when access expired
fun test_ill_02()
{
    let mut scenario_val = test_scenario::begin(PUBLISHER_ADDR);
    let scenario = &mut scenario_val;

    let mut clck = clock::create_for_testing(test_scenario::ctx(scenario));

    setup_shared_objects(test_scenario::ctx(scenario));
    test_scenario::next_tx(scenario, PUBLISHER_ADDR);
    setup_data(scenario);
    test_scenario::next_tx(scenario, PUBLISHER_ADDR);
    add_medical_record(scenario);

    test_scenario::next_tx(scenario, PATIENT_ADDR);

    {
        let mut metadata = vector::empty<String>();
        metadata.push_back(string::utf8(b"ReadAccessForAdmPersonnel"));

        create_access(
            &clck,
            ADMINISTRATIVE_PERSONNEL_ADDR,
            metadata,
            scenario
        );
    };

    test_scenario::next_tx(scenario, PROXY_ADDR);

    {
        let address_id = test_scenario::take_shared<AddressId>(scenario);
        let mut hospital_personnel_id_account = test_scenario::take_shared<HospitalPersonnelIdAccount>(scenario);
        let patient_id_account = test_scenario::take_shared<PatientIdAccount>(scenario);
        let hospital_id_metadata = test_scenario::take_shared<HospitalIdMetadata>(scenario);
        let proxy_cap = test_scenario::take_from_address<ProxyCap>(scenario, PROXY_ADDR);
        clck.set_for_testing(10 * 60 * 1000);

        let _ = get_administrative_data_test(
            &address_id,
            &clck,
            ADMINISTRATIVE_PERSONNEL_ADDR,
            &mut hospital_personnel_id_account,
            PATIENT_ADDR,
            &patient_id_account,
            &proxy_cap
        );

        test_scenario::return_shared(address_id);
        test_scenario::return_shared(hospital_personnel_id_account);
        test_scenario::return_shared(patient_id_account);
        test_scenario::return_shared(hospital_id_metadata);
        test_scenario::return_to_address(PROXY_ADDR, proxy_cap);
    };

    clck.destroy_for_testing();
    test_scenario::end(scenario_val);
}

#[test, expected_failure(abort_code = ::decmed::proxy::EAccessNotFound)]
// Read access administrative part of medical record owned by
// patient who didn't give access. By administrative personnel
fun test_ill_03()
{
    let mut scenario_val = test_scenario::begin(PUBLISHER_ADDR);
    let scenario = &mut scenario_val;

    let clck = clock::create_for_testing(test_scenario::ctx(scenario));

    setup_shared_objects(test_scenario::ctx(scenario));
    test_scenario::next_tx(scenario, PUBLISHER_ADDR);
    setup_data(scenario);
    test_scenario::next_tx(scenario, PUBLISHER_ADDR);
    add_medical_record(scenario);

    test_scenario::next_tx(scenario, PATIENT_ADDR);

    {
        let mut metadata = vector::empty<String>();
        metadata.push_back(string::utf8(b"ReadAccessForAdmPersonnel"));

        create_access(
            &clck,
            ADMINISTRATIVE_PERSONNEL_ADDR,
            metadata,
            scenario
        );
    };

    test_scenario::next_tx(scenario, PROXY_ADDR);

    {
        let address_id = test_scenario::take_shared<AddressId>(scenario);
        let mut hospital_personnel_id_account = test_scenario::take_shared<HospitalPersonnelIdAccount>(scenario);
        let patient_id_account = test_scenario::take_shared<PatientIdAccount>(scenario);
        let hospital_id_metadata = test_scenario::take_shared<HospitalIdMetadata>(scenario);
        let proxy_cap = test_scenario::take_from_address<ProxyCap>(scenario, PROXY_ADDR);

        let _ = get_administrative_data_test(
            &address_id,
            &clck,
            ADMINISTRATIVE_PERSONNEL_ADDR,
            &mut hospital_personnel_id_account,
            PATIENT_2_ADDR,
            &patient_id_account,
            &proxy_cap
        );

        test_scenario::return_shared(address_id);
        test_scenario::return_shared(hospital_personnel_id_account);
        test_scenario::return_shared(patient_id_account);
        test_scenario::return_shared(hospital_id_metadata);
        test_scenario::return_to_address(PROXY_ADDR, proxy_cap);
    };

    clck.destroy_for_testing();
    test_scenario::end(scenario_val);
}

#[test, expected_failure(abort_code = ::decmed::proxy::EAccessExpired)]
// Read access administrataive & medical part of medical record
// by medical personnel when access expired
fun test_ill_04()
{
    let mut scenario_val = test_scenario::begin(PUBLISHER_ADDR);
    let scenario = &mut scenario_val;

    let mut clck = clock::create_for_testing(test_scenario::ctx(scenario));

    setup_shared_objects(test_scenario::ctx(scenario));
    test_scenario::next_tx(scenario, PUBLISHER_ADDR);
    setup_data(scenario);
    test_scenario::next_tx(scenario, PUBLISHER_ADDR);
    add_medical_record(scenario);

    test_scenario::next_tx(scenario, PATIENT_ADDR);

    {
        let mut metadata = vector::empty<String>();
        metadata.push_back(string::utf8(b"ReadAccessForMedicalPersonnel"));
        metadata.push_back(string::utf8(b"UpdateAccessForMedicalPersonnel"));

        create_access(
            &clck,
            MEDICAL_PERSONNEL_ADDR,
            metadata,
            scenario
        );
    };

    test_scenario::next_tx(scenario, PROXY_ADDR);

    {
        let address_id = test_scenario::take_shared<AddressId>(scenario);
        let mut hospital_personnel_id_account = test_scenario::take_shared<HospitalPersonnelIdAccount>(scenario);
        let patient_id_account = test_scenario::take_shared<PatientIdAccount>(scenario);
        let hospital_id_metadata = test_scenario::take_shared<HospitalIdMetadata>(scenario);
        let proxy_cap = test_scenario::take_from_address<ProxyCap>(scenario, PROXY_ADDR);
        clck.set_for_testing(20 * 60 * 1000);

        let (_, _, _, _, _) = get_medical_record_test(
            &address_id,
            &clck,
            MEDICAL_PERSONNEL_ADDR,
            &mut hospital_personnel_id_account,
            0,
            PATIENT_ADDR,
            &patient_id_account,
            &proxy_cap
        );

        test_scenario::return_shared(address_id);
        test_scenario::return_shared(hospital_personnel_id_account);
        test_scenario::return_shared(patient_id_account);
        test_scenario::return_shared(hospital_id_metadata);
        test_scenario::return_to_address(PROXY_ADDR, proxy_cap);
    };

    clck.destroy_for_testing();
    test_scenario::end(scenario_val);
}

#[test, expected_failure(abort_code = ::decmed::proxy::EAccessNotFound)]
// Read access administrative & medical part of medical record owned by
// patient who didn't give access. By medical personnel
fun test_ill_05()
{
    let mut scenario_val = test_scenario::begin(PUBLISHER_ADDR);
    let scenario = &mut scenario_val;

    let clck = clock::create_for_testing(test_scenario::ctx(scenario));

    setup_shared_objects(test_scenario::ctx(scenario));
    test_scenario::next_tx(scenario, PUBLISHER_ADDR);
    setup_data(scenario);
    test_scenario::next_tx(scenario, PUBLISHER_ADDR);
    add_medical_record(scenario);

    test_scenario::next_tx(scenario, PATIENT_ADDR);

    {
        let mut metadata = vector::empty<String>();
        metadata.push_back(string::utf8(b"ReadAccessForMedicalPersonnel"));
        metadata.push_back(string::utf8(b"UpdateAccessForMedicalPersonnel"));

        create_access(
            &clck,
            MEDICAL_PERSONNEL_ADDR,
            metadata,
            scenario
        );
    };

    test_scenario::next_tx(scenario, PROXY_ADDR);

    {
        let address_id = test_scenario::take_shared<AddressId>(scenario);
        let mut hospital_personnel_id_account = test_scenario::take_shared<HospitalPersonnelIdAccount>(scenario);
        let patient_id_account = test_scenario::take_shared<PatientIdAccount>(scenario);
        let hospital_id_metadata = test_scenario::take_shared<HospitalIdMetadata>(scenario);
        let proxy_cap = test_scenario::take_from_address<ProxyCap>(scenario, PROXY_ADDR);

        let (_, _, _, _, _) = get_medical_record_test(
            &address_id,
            &clck,
            MEDICAL_PERSONNEL_ADDR,
            &mut hospital_personnel_id_account,
            0,
            PATIENT_2_ADDR,
            &patient_id_account,
            &proxy_cap
        );

        test_scenario::return_shared(address_id);
        test_scenario::return_shared(hospital_personnel_id_account);
        test_scenario::return_shared(patient_id_account);
        test_scenario::return_shared(hospital_id_metadata);
        test_scenario::return_to_address(PROXY_ADDR, proxy_cap);
    };

    clck.destroy_for_testing();
    test_scenario::end(scenario_val);
}

#[test, expected_failure(abort_code = ::decmed::proxy::EAccessExpired)]
// Write access a new medical record entry
// by medical personnel when access expired
fun test_ill_06()
{
    let mut scenario_val = test_scenario::begin(PUBLISHER_ADDR);
    let scenario = &mut scenario_val;

    let mut clck = clock::create_for_testing(test_scenario::ctx(scenario));

    setup_shared_objects(test_scenario::ctx(scenario));
    test_scenario::next_tx(scenario, PUBLISHER_ADDR);
    setup_data(scenario);
    test_scenario::next_tx(scenario, PUBLISHER_ADDR);
    add_medical_record(scenario);

    test_scenario::next_tx(scenario, PATIENT_ADDR);

    {
        let mut metadata = vector::empty<String>();
        metadata.push_back(string::utf8(b"ReadAccessForMedicalPersonnel"));
        metadata.push_back(string::utf8(b"UpdateAccessForMedicalPersonnel"));

        create_access(
            &clck,
            MEDICAL_PERSONNEL_ADDR,
            metadata,
            scenario
        );
    };

    test_scenario::next_tx(scenario, PROXY_ADDR);

    {
        let address_id = test_scenario::take_shared<AddressId>(scenario);
        let mut hospital_personnel_id_account = test_scenario::take_shared<HospitalPersonnelIdAccount>(scenario);
        let mut patient_id_account = test_scenario::take_shared<PatientIdAccount>(scenario);
        let hospital_id_metadata = test_scenario::take_shared<HospitalIdMetadata>(scenario);
        let proxy_cap = test_scenario::take_from_address<ProxyCap>(scenario, PROXY_ADDR);
        clck.set_for_testing(3 * 60 * 60 * 1000);

        create_medical_record_test(
            &address_id,
            &clck,
            MEDICAL_PERSONNEL_ADDR,
            &mut hospital_personnel_id_account,
            string::utf8(b"Metadata"),
            PATIENT_ADDR,
            &mut patient_id_account,
            &proxy_cap
        );

        test_scenario::return_shared(address_id);
        test_scenario::return_shared(hospital_personnel_id_account);
        test_scenario::return_shared(patient_id_account);
        test_scenario::return_shared(hospital_id_metadata);
        test_scenario::return_to_address(PROXY_ADDR, proxy_cap);
    };

    clck.destroy_for_testing();
    test_scenario::end(scenario_val);
}


#[test, expected_failure(abort_code = ::decmed::proxy::EMedicalRecordCreationLimit)]
// Write access new medical record entry twice
// by medical personnel on the same access duration
fun test_ill_07()
{
    let mut scenario_val = test_scenario::begin(PUBLISHER_ADDR);
    let scenario = &mut scenario_val;

    let clck = clock::create_for_testing(test_scenario::ctx(scenario));

    setup_shared_objects(test_scenario::ctx(scenario));
    test_scenario::next_tx(scenario, PUBLISHER_ADDR);
    setup_data(scenario);
    test_scenario::next_tx(scenario, PUBLISHER_ADDR);
    add_medical_record(scenario);

    test_scenario::next_tx(scenario, PATIENT_ADDR);

    {
        let mut metadata = vector::empty<String>();
        metadata.push_back(string::utf8(b"ReadAccessForMedicalPersonnel"));
        metadata.push_back(string::utf8(b"UpdateAccessForMedicalPersonnel"));

        create_access(
            &clck,
            MEDICAL_PERSONNEL_ADDR,
            metadata,
            scenario
        );
    };

    test_scenario::next_tx(scenario, PROXY_ADDR);

    {
        let address_id = test_scenario::take_shared<AddressId>(scenario);
        let mut hospital_personnel_id_account = test_scenario::take_shared<HospitalPersonnelIdAccount>(scenario);
        let mut patient_id_account = test_scenario::take_shared<PatientIdAccount>(scenario);
        let hospital_id_metadata = test_scenario::take_shared<HospitalIdMetadata>(scenario);
        let proxy_cap = test_scenario::take_from_address<ProxyCap>(scenario, PROXY_ADDR);

        create_medical_record_test(
            &address_id,
            &clck,
            MEDICAL_PERSONNEL_ADDR,
            &mut hospital_personnel_id_account,
            string::utf8(b"Metadata"),
            PATIENT_ADDR,
            &mut patient_id_account,
            &proxy_cap
        );

        create_medical_record_test(
            &address_id,
            &clck,
            MEDICAL_PERSONNEL_ADDR,
            &mut hospital_personnel_id_account,
            string::utf8(b"Metadata"),
            PATIENT_ADDR,
            &mut patient_id_account,
            &proxy_cap
        );

        test_scenario::return_shared(address_id);
        test_scenario::return_shared(hospital_personnel_id_account);
        test_scenario::return_shared(patient_id_account);
        test_scenario::return_shared(hospital_id_metadata);
        test_scenario::return_to_address(PROXY_ADDR, proxy_cap);
    };

    clck.destroy_for_testing();
    test_scenario::end(scenario_val);
}

#[test, expected_failure(abort_code = ::decmed::proxy::EAccessNotFound)]
// Write access of medical record owned by
// patient who didn't give access. By medical personnel
fun test_ill_08()
{
    let mut scenario_val = test_scenario::begin(PUBLISHER_ADDR);
    let scenario = &mut scenario_val;

    let clck = clock::create_for_testing(test_scenario::ctx(scenario));

    setup_shared_objects(test_scenario::ctx(scenario));
    test_scenario::next_tx(scenario, PUBLISHER_ADDR);
    setup_data(scenario);
    test_scenario::next_tx(scenario, PUBLISHER_ADDR);
    add_medical_record(scenario);

    test_scenario::next_tx(scenario, PATIENT_ADDR);

    {
        let mut metadata = vector::empty<String>();
        metadata.push_back(string::utf8(b"ReadAccessForMedicalPersonnel"));
        metadata.push_back(string::utf8(b"UpdateAccessForMedicalPersonnel"));

        create_access(
            &clck,
            MEDICAL_PERSONNEL_ADDR,
            metadata,
            scenario
        );
    };

    test_scenario::next_tx(scenario, PROXY_ADDR);

    {
        let address_id = test_scenario::take_shared<AddressId>(scenario);
        let mut hospital_personnel_id_account = test_scenario::take_shared<HospitalPersonnelIdAccount>(scenario);
        let mut patient_id_account = test_scenario::take_shared<PatientIdAccount>(scenario);
        let hospital_id_metadata = test_scenario::take_shared<HospitalIdMetadata>(scenario);
        let proxy_cap = test_scenario::take_from_address<ProxyCap>(scenario, PROXY_ADDR);

        create_medical_record_test(
            &address_id,
            &clck,
            MEDICAL_PERSONNEL_ADDR,
            &mut hospital_personnel_id_account,
            string::utf8(b"Metadata"),
            PATIENT_2_ADDR,
            &mut patient_id_account,
            &proxy_cap
        );

        test_scenario::return_shared(address_id);
        test_scenario::return_shared(hospital_personnel_id_account);
        test_scenario::return_shared(patient_id_account);
        test_scenario::return_shared(hospital_id_metadata);
        test_scenario::return_to_address(PROXY_ADDR, proxy_cap);
    };

    clck.destroy_for_testing();
    test_scenario::end(scenario_val);
}

#[test, expected_failure(abort_code = ::decmed::proxy::EAccessExpired)]
// Write access by updating a newly created record entry
// by medical personnel when access expired
fun test_ill_09()
{
    let mut scenario_val = test_scenario::begin(PUBLISHER_ADDR);
    let scenario = &mut scenario_val;

    let mut clck = clock::create_for_testing(test_scenario::ctx(scenario));

    setup_shared_objects(test_scenario::ctx(scenario));
    test_scenario::next_tx(scenario, PUBLISHER_ADDR);
    setup_data(scenario);
    test_scenario::next_tx(scenario, PUBLISHER_ADDR);
    add_medical_record(scenario);

    test_scenario::next_tx(scenario, PATIENT_ADDR);

    {
        let mut metadata = vector::empty<String>();
        metadata.push_back(string::utf8(b"ReadAccessForMedicalPersonnel"));
        metadata.push_back(string::utf8(b"UpdateAccessForMedicalPersonnel"));

        create_access(
            &clck,
            MEDICAL_PERSONNEL_ADDR,
            metadata,
            scenario
        );
    };

    test_scenario::next_tx(scenario, PROXY_ADDR);

    {
        let address_id = test_scenario::take_shared<AddressId>(scenario);
        let mut hospital_personnel_id_account = test_scenario::take_shared<HospitalPersonnelIdAccount>(scenario);
        let mut patient_id_account = test_scenario::take_shared<PatientIdAccount>(scenario);
        let hospital_id_metadata = test_scenario::take_shared<HospitalIdMetadata>(scenario);
        let proxy_cap = test_scenario::take_from_address<ProxyCap>(scenario, PROXY_ADDR);

        create_medical_record_test(
            &address_id,
            &clck,
            MEDICAL_PERSONNEL_ADDR,
            &mut hospital_personnel_id_account,
            string::utf8(b"Metadata"),
            PATIENT_ADDR,
            &mut patient_id_account,
            &proxy_cap
        );

        clck.set_for_testing(3 * 60 * 60 * 1000);

        update_medical_record_test(
            &address_id,
            &clck,
            MEDICAL_PERSONNEL_ADDR,
            &mut hospital_personnel_id_account,
            string::utf8(b"Metadata"),
            PATIENT_ADDR,
            &mut patient_id_account,
            &proxy_cap
        );

        test_scenario::return_shared(address_id);
        test_scenario::return_shared(hospital_personnel_id_account);
        test_scenario::return_shared(patient_id_account);
        test_scenario::return_shared(hospital_id_metadata);
        test_scenario::return_to_address(PROXY_ADDR, proxy_cap);
    };

    clck.destroy_for_testing();
    test_scenario::end(scenario_val);
}

#[test, expected_failure(abort_code = ::decmed::proxy::EAccessNotFound)]
// Write access by updating a newly created record entry owned by
// patient who didn't give access. By medical personnel
fun test_ill_10()
{
    let mut scenario_val = test_scenario::begin(PUBLISHER_ADDR);
    let scenario = &mut scenario_val;

    let clck = clock::create_for_testing(test_scenario::ctx(scenario));

    setup_shared_objects(test_scenario::ctx(scenario));
    test_scenario::next_tx(scenario, PUBLISHER_ADDR);
    setup_data(scenario);
    test_scenario::next_tx(scenario, PUBLISHER_ADDR);
    add_medical_record(scenario);

    test_scenario::next_tx(scenario, PATIENT_ADDR);

    {
        let mut metadata = vector::empty<String>();
        metadata.push_back(string::utf8(b"ReadAccessForMedicalPersonnel"));
        metadata.push_back(string::utf8(b"UpdateAccessForMedicalPersonnel"));

        create_access(
            &clck,
            MEDICAL_PERSONNEL_ADDR,
            metadata,
            scenario
        );
    };

    test_scenario::next_tx(scenario, PROXY_ADDR);

    {
        let address_id = test_scenario::take_shared<AddressId>(scenario);
        let mut hospital_personnel_id_account = test_scenario::take_shared<HospitalPersonnelIdAccount>(scenario);
        let mut patient_id_account = test_scenario::take_shared<PatientIdAccount>(scenario);
        let hospital_id_metadata = test_scenario::take_shared<HospitalIdMetadata>(scenario);
        let proxy_cap = test_scenario::take_from_address<ProxyCap>(scenario, PROXY_ADDR);

        create_medical_record_test(
            &address_id,
            &clck,
            MEDICAL_PERSONNEL_ADDR,
            &mut hospital_personnel_id_account,
            string::utf8(b"Metadata"),
            PATIENT_ADDR,
            &mut patient_id_account,
            &proxy_cap
        );

        update_medical_record_test(
            &address_id,
            &clck,
            MEDICAL_PERSONNEL_ADDR,
            &mut hospital_personnel_id_account,
            string::utf8(b"Metadata"),
            PATIENT_2_ADDR,
            &mut patient_id_account,
            &proxy_cap
        );

        test_scenario::return_shared(address_id);
        test_scenario::return_shared(hospital_personnel_id_account);
        test_scenario::return_shared(patient_id_account);
        test_scenario::return_shared(hospital_id_metadata);
        test_scenario::return_to_address(PROXY_ADDR, proxy_cap);
    };

    clck.destroy_for_testing();
    test_scenario::end(scenario_val);
}
