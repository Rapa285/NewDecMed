module decmed::std_struct_hospital_personnel_account;

use decmed::std_enum_hospital_personnel_role::HospitalPersonnelRole;
use decmed::std_struct_hospital_personnel_access::HospitalPersonnelAccess;
#[test_only]
use decmed::std_struct_hospital_personnel_access::default as hospital_personnel_access_default;
use decmed::std_struct_hospital_personnel_administrative_metadata::HospitalPersonnelAdministrativeMetadata;
#[test_only]
use decmed::std_struct_hospital_personnel_administrative_metadata::default as hospital_personnel_administrative_metadata_default;
use decmed::std_struct_hospital_personnel_metadata::HospitalPersonnelMetadata;

use iota::vec_map::VecMap;

#[test_only]
use std::string::Self;
use std::string::String;

/// - `activation_key`: argon_hash(<raw_uuid_v4>@<raw_id>)
/// - `hospital_id`: encoded argon_hash(raw_hospital_id)
/// - `personnels`: <K: hospital_personnel_id>
public struct HospitalPersonnelAccount has copy, drop, store {
    access: Option<HospitalPersonnelAccess>,
    activation_key: String,
    address: Option<address>,
    administrative_metadata: Option<HospitalPersonnelAdministrativeMetadata>,
    hospital_id: String,
    is_activation_key_used: bool,
    is_profile_completed: bool,
    personnels: Option<VecMap<String, HospitalPersonnelMetadata>>,
    role: HospitalPersonnelRole,
}

public(package) fun new(
    access: Option<HospitalPersonnelAccess>,
    activation_key: String,
    address: Option<address>,
    administrative_metadata: Option<HospitalPersonnelAdministrativeMetadata>,
    hospital_id: String,
    is_activation_key_used: bool,
    is_profile_completed: bool,
    personnels: Option<VecMap<String, HospitalPersonnelMetadata>>,
    role: HospitalPersonnelRole,
): HospitalPersonnelAccount
{
    HospitalPersonnelAccount {
    	access,
    	activation_key,
    	address,
    	administrative_metadata,
    	hospital_id,
    	is_activation_key_used,
        is_profile_completed,
    	personnels,
    	role,
    }
}

public(package) fun borrow_access(
    self: &HospitalPersonnelAccount,
): &Option<HospitalPersonnelAccess>
{
    &self.access
}

public(package) fun borrow_mut_access(
    self: &mut HospitalPersonnelAccount,
): &mut Option<HospitalPersonnelAccess>
{
    &mut self.access
}

public(package) fun set_access(
    self: &mut HospitalPersonnelAccount,
    access: Option<HospitalPersonnelAccess>,
)
{
    self.access = access;
}

public(package) fun borrow_activation_key(
    self: &HospitalPersonnelAccount,
): &String
{
    &self.activation_key
}

public(package) fun borrow_mut_activation_key(
    self: &mut HospitalPersonnelAccount,
): &mut String
{
    &mut self.activation_key
}

public(package) fun set_activation_key(
    self: &mut HospitalPersonnelAccount,
    activation_key: String,
)
{
    self.activation_key = activation_key;
}

public(package) fun borrow_address(
    self: &HospitalPersonnelAccount,
): &Option<address>
{
    &self.address
}

public(package) fun borrow_mut_address(
    self: &mut HospitalPersonnelAccount,
): &mut Option<address>
{
    &mut self.address
}

public(package) fun set_address(
    self: &mut HospitalPersonnelAccount,
    address: Option<address>,
)
{
    self.address = address;
}

public(package) fun borrow_administrative_metadata(
    self: &HospitalPersonnelAccount,
): &Option<HospitalPersonnelAdministrativeMetadata>
{
    &self.administrative_metadata
}

public(package) fun borrow_mut_administrative_metadata(
    self: &mut HospitalPersonnelAccount,
): &mut Option<HospitalPersonnelAdministrativeMetadata>
{
    &mut self.administrative_metadata
}

public(package) fun set_administrative_metadata(
    self: &mut HospitalPersonnelAccount,
    administrative_metadata: Option<HospitalPersonnelAdministrativeMetadata>,
)
{
    self.administrative_metadata = administrative_metadata;
}

public(package) fun borrow_hospital_id(
    self: &HospitalPersonnelAccount,
): &String
{
    &self.hospital_id
}

public(package) fun borrow_mut_hospital_id(
    self: &mut HospitalPersonnelAccount,
): &mut String
{
    &mut self.hospital_id
}

public(package) fun set_hospital_id(
    self: &mut HospitalPersonnelAccount,
    hospital_id: String,
)
{
    self.hospital_id = hospital_id;
}

public(package) fun borrow_is_activation_key_used(
    self: &HospitalPersonnelAccount,
): bool
{
    self.is_activation_key_used
}

public(package) fun set_is_activation_key_used(
    self: &mut HospitalPersonnelAccount,
    is_activation_key_used: bool,
)
{
    self.is_activation_key_used = is_activation_key_used;
}

public(package) fun borrow_is_profile_completed(
    self: &HospitalPersonnelAccount,
): bool
{
    self.is_profile_completed
}

public(package) fun set_is_profile_completed(
    self: &mut HospitalPersonnelAccount,
    is_profile_completed: bool,
)
{
    self.is_profile_completed = is_profile_completed;
}

public(package) fun borrow_personnels(
    self: &HospitalPersonnelAccount,
): &Option<VecMap<String, HospitalPersonnelMetadata>>
{
    &self.personnels
}

public(package) fun borrow_mut_personnels(
    self: &mut HospitalPersonnelAccount,
): &mut Option<VecMap<String, HospitalPersonnelMetadata>>
{
    &mut self.personnels
}

public(package) fun set_personnels(
    self: &mut HospitalPersonnelAccount,
    personnels: Option<VecMap<String, HospitalPersonnelMetadata>>,
)
{
    self.personnels = personnels;
}

public(package) fun borrow_role(
    self: &HospitalPersonnelAccount,
): &HospitalPersonnelRole
{
    &self.role
}

public(package) fun borrow_mut_role(
    self: &mut HospitalPersonnelAccount,
): &mut HospitalPersonnelRole
{
    &mut self.role
}

public(package) fun set_role(
    self: &mut HospitalPersonnelAccount,
    role: HospitalPersonnelRole,
)
{
    self.role = role;
}

#[test_only]
public(package) fun default(role: HospitalPersonnelRole): HospitalPersonnelAccount
{
    HospitalPersonnelAccount {
    	access: option::some(hospital_personnel_access_default()),
    	activation_key: string::utf8(b"ActivationKey"),
    	address: option::none(),
    	administrative_metadata: option::some(hospital_personnel_administrative_metadata_default()),
    	hospital_id: string::utf8(b"Hos1"),
    	is_activation_key_used: false,
    	is_profile_completed: false,
    	personnels: option::none(),
    	role,
    }
}
