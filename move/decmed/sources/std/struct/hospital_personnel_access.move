module decmed::std_struct_hospital_personnel_access;

use decmed::std_struct_hospital_personnel_access_data::HospitalPersonnelAccessData;

#[test_only]
use iota::vec_map::Self;
use iota::vec_map::VecMap;

use std::string::String;

public struct HospitalPersonnelAccess has copy, drop, store {
    read: VecMap<String, HospitalPersonnelAccessData>,
    update: VecMap<String, HospitalPersonnelAccessData>,
}

public(package) fun new(
    read: VecMap<String, HospitalPersonnelAccessData>,
    update: VecMap<String, HospitalPersonnelAccessData>,
): HospitalPersonnelAccess
{
    HospitalPersonnelAccess { read, update }
}

public(package) fun borrow_read(
    self: &HospitalPersonnelAccess,
): &VecMap<String, HospitalPersonnelAccessData>
{
    &self.read
}

public(package) fun borrow_mut_read(
    self: &mut HospitalPersonnelAccess,
): &mut VecMap<String, HospitalPersonnelAccessData>
{
    &mut self.read
}

public(package) fun set_read(
    self: &mut HospitalPersonnelAccess,
    read: VecMap<String, HospitalPersonnelAccessData>,
)
{
    self.read = read;
}

public(package) fun borrow_update(
    self: &HospitalPersonnelAccess,
): &VecMap<String, HospitalPersonnelAccessData>
{
    &self.update
}

public(package) fun borrow_mut_update(
    self: &mut HospitalPersonnelAccess,
): &mut VecMap<String, HospitalPersonnelAccessData>
{
    &mut self.update
}

public(package) fun set_update(
    self: &mut HospitalPersonnelAccess,
    update: VecMap<String, HospitalPersonnelAccessData>,
)
{
    self.update = update;
}

#[test_only]
public(package) fun default(): HospitalPersonnelAccess
{
    HospitalPersonnelAccess {
        read: vec_map::empty<String, HospitalPersonnelAccessData>(),
        update: vec_map::empty<String, HospitalPersonnelAccessData>(),
    }
}
