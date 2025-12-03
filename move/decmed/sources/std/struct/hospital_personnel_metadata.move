module decmed::std_struct_hospital_personnel_metadata;

use std::string::String;

/// - `metadata`: Base64 encoded
public struct HospitalPersonnelMetadata has copy, drop, store {
    metadata: String,
}

public(package) fun new(
    metadata: String,
): HospitalPersonnelMetadata
{
    HospitalPersonnelMetadata { metadata }
}

public(package) fun borrow_metadata(
    self: &HospitalPersonnelMetadata,
): &String
{
    &self.metadata
}

public(package) fun borrow_mut_metadata(
    self: &mut HospitalPersonnelMetadata,
): &mut String
{
    &mut self.metadata
}

public(package) fun set_metadata(
    self: &mut HospitalPersonnelMetadata,
    metadata: String,
)
{
    self.metadata = metadata;
}
