module decmed::std_struct_hospital_personnel_access_data;

use decmed::std_enum_hospital_personnel_access_data_type::HospitalPersonnelAccessDataType;

#[test_only]
use std::string::Self;
use std::string::String;

public struct HospitalPersonnelAccessData has copy, drop, store {
    access_data_types: vector<HospitalPersonnelAccessDataType>,
    exp: u64,
    metadata: String,
    medical_metadata_index: Option<u64>,
}

public(package) fun new(
    access_data_types: vector<HospitalPersonnelAccessDataType>,
    exp: u64,
    metadata: String,
    medical_metadata_index: Option<u64>,
): HospitalPersonnelAccessData
{
    HospitalPersonnelAccessData {
        access_data_types,
        exp,
        metadata,
        medical_metadata_index
    }
}

public(package) fun borrow_access_data_types(
    self: &HospitalPersonnelAccessData,
): &vector<HospitalPersonnelAccessDataType>
{
    &self.access_data_types
}

public(package) fun borrow_mut_access_data_types(
    self: &mut HospitalPersonnelAccessData,
): &mut vector<HospitalPersonnelAccessDataType>
{
    &mut self.access_data_types
}

public(package) fun set_access_data_types(
    self: &mut HospitalPersonnelAccessData,
    access_data_types: vector<HospitalPersonnelAccessDataType>,
)
{
    self.access_data_types = access_data_types;
}

public(package) fun borrow_exp(
    self: &HospitalPersonnelAccessData,
): u64
{
    self.exp
}

public(package) fun set_exp(
    self: &mut HospitalPersonnelAccessData,
    exp: u64,
)
{
    self.exp = exp;
}

public(package) fun borrow_medical_metadata_index(
    self: &HospitalPersonnelAccessData,
): Option<u64>
{
    self.medical_metadata_index
}

public(package) fun set_medical_metadata_index(
    self: &mut HospitalPersonnelAccessData,
    medical_metadata_index: Option<u64>,
)
{
    self.medical_metadata_index = medical_metadata_index;
}

#[test_only]
public(package) fun default(): HospitalPersonnelAccessData
{
    HospitalPersonnelAccessData {
    	access_data_types: vector::empty<HospitalPersonnelAccessDataType>(),
    	exp: 0,
    	metadata: string::utf8(b"Metadata"),
    	medical_metadata_index: option::none(),
    }
}
