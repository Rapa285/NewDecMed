module decmed::std_struct_patient_account;

use decmed::std_struct_patient_access_log::PatientAccessLog;
use decmed::std_struct_patient_administrative_metadata::PatientAdministrativeMetadata;
#[test_only]
use decmed::std_struct_patient_administrative_metadata::default as patient_administrative_metadata_default;
use decmed::std_struct_patient_medical_metadata::PatientMedicalMetadata;

#[test_only]
use iota::table_vec::Self;
use iota::table_vec::TableVec;

public struct PatientAccount has store {
    access_log: TableVec<PatientAccessLog>,
    address: address,
    administrative_metadata: PatientAdministrativeMetadata,
    is_profile_completed: bool,
    medical_metadata: TableVec<PatientMedicalMetadata>,
}

public(package) fun new(
    access_log: TableVec<PatientAccessLog>,
    address: address,
    administrative_metadata: PatientAdministrativeMetadata,
    is_profile_completed: bool,
    medical_metadata: TableVec<PatientMedicalMetadata>,
): PatientAccount
{
    PatientAccount {
    	access_log,
        address,
    	administrative_metadata,
        is_profile_completed,
    	medical_metadata,
    }
}

public(package) fun borrow_access_log(
    self: &PatientAccount,
): &TableVec<PatientAccessLog>
{
    &self.access_log
}

public(package) fun borrow_mut_access_log(
    self: &mut PatientAccount,
): &mut TableVec<PatientAccessLog>
{
    &mut self.access_log
}

public(package) fun borrow_administrative_metadata(
    self: &PatientAccount,
): &PatientAdministrativeMetadata
{
    &self.administrative_metadata
}

public(package) fun borrow_mut_administrative_metadata(
    self: &mut PatientAccount,
): &mut PatientAdministrativeMetadata
{
    &mut self.administrative_metadata
}

public(package) fun set_administrative_metadata(
    self: &mut PatientAccount,
    administrative_metadata: PatientAdministrativeMetadata,
)
{
    self.administrative_metadata = administrative_metadata;
}

public(package) fun borrow_is_profile_completed(
    self: &PatientAccount,
): bool
{
    self.is_profile_completed
}

public(package) fun set_is_profile_completed(
    self: &mut PatientAccount,
    is_profile_completed: bool,
)
{
    self.is_profile_completed = is_profile_completed;
}

public(package) fun borrow_medical_metadata(
    self: &PatientAccount,
): &TableVec<PatientMedicalMetadata>
{
    &self.medical_metadata
}

public(package) fun borrow_mut_medical_metadata(
    self: &mut PatientAccount,
): &mut TableVec<PatientMedicalMetadata>
{
    &mut self.medical_metadata
}

#[test_only]
public(package) fun default(
    address: address,
    ctx: &mut TxContext,
): PatientAccount
{
    PatientAccount {
    	access_log: table_vec::empty<PatientAccessLog>(ctx),
    	address,
    	administrative_metadata: patient_administrative_metadata_default(),
    	is_profile_completed: false,
    	medical_metadata: table_vec::empty<PatientMedicalMetadata>(ctx),
    }
}
