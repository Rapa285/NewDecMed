module decmed::std_struct_patient_administrative_metadata;

#[test_only]
use std::string::Self;
use std::string::String;

public struct PatientAdministrativeMetadata has copy, drop, store {
    private_metadata: String,
}

public(package) fun new(
    private_metadata: String,
): PatientAdministrativeMetadata
{
    PatientAdministrativeMetadata { private_metadata }
}

public(package) fun borrow_private_metadata(
    self: &PatientAdministrativeMetadata,
): &String
{
    &self.private_metadata
}

public(package) fun borrow_mut_private_metadata(
    self: &mut PatientAdministrativeMetadata,
): &mut String
{
    &mut self.private_metadata
}

public(package) fun set_private_metadata(
    self: &mut PatientAdministrativeMetadata,
    private_metadata: String,
)
{
    self.private_metadata = private_metadata;
}

#[test_only]
public(package) fun default(): PatientAdministrativeMetadata
{
    PatientAdministrativeMetadata {
        private_metadata: string::utf8(b"PatientPrivateMetadata")
    }
}
