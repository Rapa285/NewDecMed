module decmed::std_struct_hospital_personnel_administrative_metadata;

#[test_only]
use std::string::Self;
use std::string::String;

public struct HospitalPersonnelAdministrativeMetadata has copy, drop, store {
    private_metadata: String,
    public_metadata: String,
}

public(package) fun new(
    private_metadata: String,
    public_metadata: String,
): HospitalPersonnelAdministrativeMetadata
{
    HospitalPersonnelAdministrativeMetadata { private_metadata, public_metadata }
}

public(package) fun borrow_private_metadata(
    self: &HospitalPersonnelAdministrativeMetadata,
): &String
{
    &self.private_metadata
}

public(package) fun borrow_mut_private_metadata(
    self: &mut HospitalPersonnelAdministrativeMetadata,
): &mut String
{
    &mut self.private_metadata
}

public(package) fun set_private_metadata(
    self: &mut HospitalPersonnelAdministrativeMetadata,
    private_metadata: String,
)
{
    self.private_metadata = private_metadata;
}

public(package) fun borrow_public_metadata(
    self: &HospitalPersonnelAdministrativeMetadata,
): &String
{
    &self.public_metadata
}

public(package) fun borrow_mut_public_metadata(
    self: &mut HospitalPersonnelAdministrativeMetadata,
): &mut String
{
    &mut self.public_metadata
}

public(package) fun set_public_metadata(
    self: &mut HospitalPersonnelAdministrativeMetadata,
    public_metadata: String,
)
{
    self.public_metadata = public_metadata;
}

#[test_only]
public(package) fun default(): HospitalPersonnelAdministrativeMetadata
{
    HospitalPersonnelAdministrativeMetadata {
        private_metadata: string::utf8(b"HospitalPersonnelPrivateMetadata"),
        public_metadata: string::utf8(b"HospitalPersonnelPublicMetadata")
    }
}
