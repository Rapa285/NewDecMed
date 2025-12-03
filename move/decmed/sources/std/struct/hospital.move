module decmed::std_struct_hospital;

use decmed::std_struct_hospital_metadata::HospitalMetadata;

use std::string::String;

public struct Hospital has copy, drop, store {
    admin_metadata: String,
    hospital_metadata: HospitalMetadata,
}

public(package) fun new(
    admin_metadata: String,
    hospital_metadata: HospitalMetadata,
): Hospital
{
    Hospital {
        admin_metadata,
        hospital_metadata,
    }
}

public(package) fun borrow_admin_metadata(
    self: &Hospital,
): &String
{
    &self.admin_metadata
}

public(package) fun borrow_mut_admin_metadata(
    self: &mut Hospital,
): &mut String
{
    &mut self.admin_metadata
}

public(package) fun set_admin_metadata(
    self: &mut Hospital,
    admin_metadata: String,
)
{
    self.admin_metadata = admin_metadata;
}

public(package) fun borrow_hospital_metadata(
    self: &Hospital,
): &HospitalMetadata
{
    &self.hospital_metadata
}

public(package) fun borrow_mut_hospital_metadata(
    self: &mut Hospital,
): &mut HospitalMetadata
{
    &mut self.hospital_metadata
}

public(package) fun set_hospital_metadata(
    self: &mut Hospital,
    hospital_metadata: HospitalMetadata,
)
{
    self.hospital_metadata = hospital_metadata;
}
