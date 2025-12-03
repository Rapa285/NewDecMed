module decmed::std_struct_hospital_admin_metadata;

use std::string::String;

public struct HospitalAdminMetadata has copy, drop, store {
    metadata: String,
}

public(package) fun new(
    metadata: String,
):HospitalAdminMetadata
{
    HospitalAdminMetadata {
        metadata
    }
}

public(package) fun borrow_metadata(
    self: &HospitalAdminMetadata,
): &String
{
    &self.metadata
}

public(package) fun borrow_mut_metadata(
    self: &mut HospitalAdminMetadata,
): &mut String
{
    &mut self.metadata
}

public(package) fun set_metadata(
    self: &mut HospitalAdminMetadata,
    metadata: String,
)
{
    self.metadata = metadata;
}
