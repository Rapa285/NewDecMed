module decmed::std_struct_hospital_metadata;

use std::string::String;

public struct HospitalMetadata has copy, drop, store {
    name: String,
}

public(package) fun new(
    name: String,
): HospitalMetadata
{
    HospitalMetadata {
        name,
    }
}

public(package) fun borrow_name(
    self: &HospitalMetadata,
): &String
{
    &self.name
}

public(package) fun borrow_mut_name(
    self: &mut HospitalMetadata,
): &mut String
{
    &mut self.name
}

public(package) fun set_name(
    self: &mut HospitalMetadata,
    name: String,
)
{
    self.name = name;
}
