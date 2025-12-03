module decmed::hospital_personnel;

use decmed::std_enum_hospital_personnel_role::{
    HospitalPersonnelRole,
    admin as hospital_personnel_role_admin,
    administrative_personnel as hospital_personnel_role_admin_administrative_personnel,
    medical_personnel as hospital_personnel_role_medical_personnel,
};

use decmed::shared::{
    encode_hospital_id,
    encode_hospital_personnel_id,
};

use decmed::std_struct_address_id::AddressId;
use decmed::std_struct_hospital_id_metadata::HospitalIdMetadata;
use decmed::std_struct_hospital_metadata::HospitalMetadata;
use decmed::std_struct_hospital_personnel_access::{
    new as hospital_personnel_access_new,
};
use decmed::std_struct_hospital_personnel_access_data::HospitalPersonnelAccessData;
use decmed::std_struct_hospital_personnel_account::{
    HospitalPersonnelAccount,
    new as hospital_personnel_account_new,
};
use decmed::std_struct_hospital_personnel_administrative_metadata::{
    HospitalPersonnelAdministrativeMetadata,
    new as hospital_personnel_administrative_metadata_new,
};
use decmed::std_struct_hospital_personnel_id_account::HospitalPersonnelIdAccount;
use decmed::std_struct_hospital_personnel_metadata::{
    HospitalPersonnelMetadata,
    new as hospital_personnel_metadata_new,
};
use decmed::std_struct_patient_id_account::PatientIdAccount;
use decmed::std_struct_patient_medical_metadata::{
    new as patient_medical_metadata_new,
};

use iota::clock::Clock;
use iota::vec_map;

use std::string::{String};

// Constants

const EAccountAlreadyRegistered: u64 = 2000;
const EAccountNotActivated: u64 = 2001;
const EAccountNotFound: u64 = 2002;
const EActivationKeyAlreadyUsed: u64 = 2003;
const EAddressAlreadyRegistered: u64 = 2004;
const EIllegalActionAccessExpired: u64 = 2005;
const EIllegalActionInvalidRole: u64 = 2006;
const EIllegalActionNoUpdateAccess: u64 = 2007;
const EInvalidActivationKey: u64 = 2008;
const EInvalidHospitalPersonnelRole: u64 = 2009;
const EPatientNotFound: u64 = 2010;

// Functions

/// ## Params
/// - `activation_key`: argon_hash(<raw_uuid_v4>@<raw_id>)
entry fun cleanup_read_access(
    activation_key: String,
    address_id: &AddressId,
    clock: &Clock,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    ctx: &TxContext,
)
{
    let address_id_table = address_id.borrow_table();
    let hospital_personnel_id = *address_id_table.borrow(ctx.sender());
    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_mut_table();
    let hospital_personnel_account = hospital_personnel_id_account_table.borrow_mut(hospital_personnel_id);

    require_account_activation(activation_key, hospital_personnel_account);

    let hospital_personnel_access = hospital_personnel_account.borrow_mut_access().borrow_mut();
    let hospital_personnel_read_access = hospital_personnel_access.borrow_read();

    let mut cnt = 0;
    let len = hospital_personnel_read_access.size();
    let current_time = clock.timestamp_ms();

    let mut hospital_personnel_read_access_new = vec_map::empty<String, HospitalPersonnelAccessData>();

    while (cnt < len) {
        let (patient_id, access) = hospital_personnel_read_access.get_entry_by_idx(cnt);

        if (access.borrow_exp() < current_time) {
            cnt = cnt + 1;
            continue
        };

        hospital_personnel_read_access_new.insert(*patient_id, *access);
        cnt = cnt + 1;
    };

    hospital_personnel_access.set_read(hospital_personnel_read_access_new);
}

/// ## Params
/// - `activation_key`: argon_hash(<raw_uuid_v4>@<raw_id>)
entry fun cleanup_update_access(
    activation_key: String,
    address_id: &AddressId,
    clock: &Clock,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    ctx: &TxContext,
)
{
    let address_id_table = address_id.borrow_table();
    let hospital_personnel_id = *address_id_table.borrow(ctx.sender());
    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_mut_table();
    let hospital_personnel_account = hospital_personnel_id_account_table.borrow_mut(hospital_personnel_id);

    require_account_activation(activation_key, hospital_personnel_account);

    let hospital_personnel_access = hospital_personnel_account.borrow_mut_access().borrow_mut();
    let hospital_personnel_update_access = hospital_personnel_access.borrow_update();

    let mut cnt = 0;
    let len = hospital_personnel_update_access.size();
    let current_time = clock.timestamp_ms();

    let mut hospital_personnel_update_access_new = vec_map::empty<String, HospitalPersonnelAccessData>();

    while (cnt < len) {
        let (patient_id, access) = hospital_personnel_update_access.get_entry_by_idx(cnt);

        if (access.borrow_exp() < current_time) {
            cnt = cnt + 1;
            continue
        };

        hospital_personnel_update_access_new.insert(*patient_id, *access);
        cnt = cnt + 1;
    };

    hospital_personnel_access.set_update(hospital_personnel_update_access_new);
}

/// ## Params
/// - `activation_key`: argon_hash(<raw_uuid_v4>@<raw_id>)
/// - `id`: argon_hash(raw_id)
/// - `metadata`: Base64
/// - `role`: raw_role ["Admin", "AdministrativePersonnel", "MedicalPersonnel"]
entry fun create_activation_key(
    address_id: &AddressId,
    admin_activation_key: String,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    metadata: String,
    personnel_activation_key: String,
    personnel_id: String,
    role: vector<u8>,
    ctx: &TxContext,
)
{
    let address_id_table = address_id.borrow_table();
    let hospital_admin_id = *address_id_table.borrow(ctx.sender());
    let hopsital_personnel_id_account_table = hospital_personnel_id_account.borrow_mut_table();
    let hospital_admin_account = hopsital_personnel_id_account_table.borrow(hospital_admin_id);

    require_account_activation(admin_activation_key, hospital_admin_account);

    assert!(hospital_admin_account.borrow_role() == hospital_personnel_role_admin(), EIllegalActionInvalidRole);

    let hospital_personnel_id = encode_hospital_personnel_id(*hospital_admin_account.borrow_hospital_id(), personnel_id);

    assert!(!hopsital_personnel_id_account_table.contains(hospital_personnel_id), EAccountAlreadyRegistered);

    let role = match (role) {
        b"AdministrativePersonnel" => hospital_personnel_role_admin_administrative_personnel(),
        b"MedicalPersonnel" => hospital_personnel_role_medical_personnel(),
        _ => return assert!(false, EInvalidHospitalPersonnelRole)
    };

    let account = hospital_personnel_account_new(
        option::none(),
        personnel_activation_key,
        option::none(),
        option::none(),
        *hospital_admin_account.borrow_hospital_id(),
        false,
        false,
        option::none(),
        role
    );

    hopsital_personnel_id_account_table.add(hospital_personnel_id, account);

    let hospital_admin_account = hopsital_personnel_id_account_table.borrow_mut(hospital_admin_id);

    let hospital_admin_account_personnels = hospital_admin_account.borrow_mut_personnels().borrow_mut();
    let personnel_metadata = hospital_personnel_metadata_new(metadata);

    hospital_admin_account_personnels.insert(hospital_personnel_id, personnel_metadata);
}

#[test_only]
public(package) fun create_activation_key_test(
    address_id: &AddressId,
    admin_activation_key: String,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    metadata: String,
    personnel_activation_key: String,
    personnel_id: String,
    role: vector<u8>,
    ctx: &TxContext,
)
{
    create_activation_key(
        address_id,
        admin_activation_key,
        hospital_personnel_id_account,
        metadata,
        personnel_activation_key,
        personnel_id,
        role,
        ctx
    );
}

/// ## Params:
/// - `activation_key`: argon_hash(<raw_uuid_v4>@<raw_id>)
/// - `metadata`: Base64 decoded string
entry fun create_medical_record(
    activation_key: String,
    address_id: &AddressId,
    clock: &Clock,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    metadata: String,
    patient_address: address,
    patient_id_account: &mut PatientIdAccount,
    ctx: &TxContext,
)
{
    let address_id_table = address_id.borrow_table();

    assert!(address_id_table.contains(patient_address), EPatientNotFound);

    let patient_id = address_id_table.borrow(patient_address);

    let hospital_personnel_id = *address_id_table.borrow(ctx.sender());
    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_mut_table();
    let hospital_personnel_account = hospital_personnel_id_account_table.borrow_mut(hospital_personnel_id);

    require_account_activation(activation_key, hospital_personnel_account);

    let hospital_personnel_access = hospital_personnel_account.borrow_mut_access().borrow_mut();
    let hospital_personnel_update_access = hospital_personnel_access.borrow_mut_update();

    assert!(hospital_personnel_update_access.contains(patient_id), EIllegalActionNoUpdateAccess);

    let access = hospital_personnel_update_access.get(patient_id);

    if (access.borrow_exp() < clock.timestamp_ms()) {
        hospital_personnel_update_access.remove(patient_id);
        assert!(false, EIllegalActionAccessExpired);
    };

    let patient_id_account_table = patient_id_account.borrow_mut_table();
    let patient_account = patient_id_account_table.borrow_mut(*patient_id);
    let patient_medical_metadata = patient_account.borrow_mut_medical_metadata();

    let medical_metadata = patient_medical_metadata_new(patient_medical_metadata.length(), metadata);
    patient_medical_metadata.push_back(medical_metadata);
}

/// ## Params
/// - `activation_key`: argon_hash(<raw_uuid_v4>@<raw_id>)
/// - `hospital_id`: argon_hash(raw_hospital_id)
/// - `personel_id`: argon_hash(raw_id)
entry fun delete_hospital_personnel(
    activation_key: String,
    address_id: &mut AddressId,
    hospital_id: String,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    personnel_id: String,
    ctx: &TxContext,
)
{
    let address_id_table = address_id.borrow_mut_table();

    let hospital_admin_id = *address_id_table.borrow(ctx.sender());

    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_mut_table();
    let hospital_admin_account = hospital_personnel_id_account_table.borrow(hospital_admin_id);

    assert!(hospital_admin_account.borrow_role() == hospital_personnel_role_admin(), EIllegalActionInvalidRole);
    require_account_activation(activation_key, hospital_admin_account);

    let hospital_id = encode_hospital_id(hospital_id);
    let hospital_personnel_id = encode_hospital_personnel_id(hospital_id, personnel_id);
    let hospital_personnel_account = hospital_personnel_id_account_table.borrow(hospital_personnel_id);

    address_id_table.remove(*hospital_personnel_account.borrow_address().borrow());

    let hospital_admin_account = hospital_personnel_id_account_table.borrow_mut(hospital_admin_id);
    let hospital_admin_personnels = hospital_admin_account.borrow_mut_personnels().borrow_mut();
    assert!(hospital_admin_personnels.contains(&hospital_personnel_id), EAccountNotFound);

    hospital_admin_personnels.remove(&hospital_personnel_id);
    hospital_personnel_id_account_table.remove(hospital_personnel_id);
}

/// ## Params
/// - `activation_key`: argon_hash(<raw_uuid_v4>@<raw_id>)
entry fun get_account_info(
    activation_key: String,
    address_id: &AddressId,
    hospital_id_metadata: &HospitalIdMetadata,
    hospital_personnel_id_account: &HospitalPersonnelIdAccount,
    ctx: &TxContext,
): (Option<HospitalPersonnelAdministrativeMetadata>, HospitalPersonnelRole, HospitalMetadata)
{
    let address_id_table = address_id.borrow_table();
    let hospital_personnel_id = *address_id_table.borrow(ctx.sender());
    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_table();
    let hospital_personnel_account = hospital_personnel_id_account_table.borrow(hospital_personnel_id);

    require_account_activation(activation_key, hospital_personnel_account);

    let hospital_id_metadata_table = hospital_id_metadata.borrow_table();
    let hospital_id_metadata_vec = hospital_id_metadata.borrow_vec();
    let hospital_metadata_index = *hospital_id_metadata_table.borrow(*hospital_personnel_account.borrow_hospital_id());
    let hospital_metadata = hospital_id_metadata_vec.borrow(hospital_metadata_index).borrow_hospital_metadata();

    (*hospital_personnel_account.borrow_administrative_metadata(), *hospital_personnel_account.borrow_role(), *hospital_metadata)
}


/// ## Params
/// - `activation_key`: argon_hash(<raw_uuid_v4>@<raw_id>)
/// - `hospital_id`: argon_hash(raw_hospital_id)
/// - `personel_id`: argon_hash(raw_id)
///
/// ## Return:
/// 0: state_code
///     - 0 means need activation
///     - 1 means need signup
///     - 2 means need profile completion
///     - 3 means no action
entry fun get_account_state(
    activation_key: String,
    hospital_id: String,
    hospital_personnel_id_account: &HospitalPersonnelIdAccount,
    personnel_id: String,
): (u64, Option<HospitalPersonnelRole>)
{
    let hospital_id = encode_hospital_id(hospital_id);
    let hospital_personnel_id = encode_hospital_personnel_id(hospital_id, personnel_id);
    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_table();

    if (!hospital_personnel_id_account_table.contains(hospital_personnel_id)) {
        return (0, option::none())
    };

    let hospital_personnel_account = hospital_personnel_id_account_table.borrow(hospital_personnel_id);

    if (*hospital_personnel_account.borrow_activation_key() != activation_key || !hospital_personnel_account.borrow_is_activation_key_used()) {
        return (0, option::none())
    };

    if (hospital_personnel_account.borrow_address().is_none()) {
        return (1, option::none())
    };

    if (!hospital_personnel_account.borrow_is_profile_completed()) {
        return (2, option::none())
    };

    (3, option::some(*hospital_personnel_account.borrow_role()))
}

/// ## Params
/// - `activation_key`: argon_hash(<raw_uuid_v4>@<raw_id>)
entry fun get_hospital_personnels(
    activation_key: String,
    address_id: &AddressId,
    hospital_personnel_id_account: &HospitalPersonnelIdAccount,
    ctx: &TxContext,
): vector<HospitalPersonnelMetadata>
{
    let address_id_table = address_id.borrow_table();
    let hospital_admin_id = *address_id_table.borrow(ctx.sender());

    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_table();
    let hospital_admin_account = hospital_personnel_id_account_table.borrow(hospital_admin_id);

    require_account_activation(activation_key, hospital_admin_account);

    let hospital_admin_personnels = *hospital_admin_account.borrow_personnels().borrow();
    let (_, personnels) = hospital_admin_personnels.into_keys_values();

    personnels
}

/// ## Params
/// - `activation_key`: argon_hash(<raw_uuid_v4>@<raw_id>)
entry fun get_read_access(
    activation_key: String,
    address_id: &AddressId,
    hospital_personnel_id_account: &HospitalPersonnelIdAccount,
    ctx: &TxContext,
): vector<HospitalPersonnelAccessData>
{
    let address_id_table = address_id.borrow_table();
    let hospital_personnel_id = *address_id_table.borrow(ctx.sender());
    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_table();
    let hospital_personnel_account = hospital_personnel_id_account_table.borrow(hospital_personnel_id);

    require_account_activation(activation_key, hospital_personnel_account);

    let hospital_personnel_access = hospital_personnel_account.borrow_access().borrow();
    let hospital_personnel_read_access = *hospital_personnel_access.borrow_read();
    let (_, res) = hospital_personnel_read_access.into_keys_values();

    res
}

/// ## Params
/// - `activation_key`: argon_hash(<raw_uuid_v4>@<raw_id>)
entry fun get_update_access(
    activation_key: String,
    address_id: &AddressId,
    hospital_personnel_id_account: &HospitalPersonnelIdAccount,
    ctx: &TxContext,
): vector<HospitalPersonnelAccessData>
{
    let address_id_table = address_id.borrow_table();
    let hospital_personnel_id = *address_id_table.borrow(ctx.sender());
    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_table();
    let hospital_personnel_account = hospital_personnel_id_account_table.borrow(hospital_personnel_id);

    require_account_activation(activation_key, hospital_personnel_account);

    let hospital_personnel_access = hospital_personnel_account.borrow_access().borrow();
    let hospital_personnel_update_access = *hospital_personnel_access.borrow_update();
    let (_, res) = hospital_personnel_update_access.into_keys_values();

    res
}

entry fun is_account_registered(
    activation_key: String,
    address_id: &AddressId,
    hospital_personnel_id_account: &HospitalPersonnelIdAccount,
    ctx: &TxContext,
)
{
    let address_id_table = address_id.borrow_table();
    let hospital_personnel_id = *address_id_table.borrow(ctx.sender());
    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_table();
    let hospital_personnel_account = hospital_personnel_id_account_table.borrow(hospital_personnel_id);

    require_account_activation(activation_key, hospital_personnel_account);
}

/// ## Params
/// - `activation_key`: argon_hash(<raw_uuid_v4>@<raw_id>)
fun require_account_activation(
    activation_key: String,
    hospital_personnel_account: &HospitalPersonnelAccount,
)
{
    assert!(hospital_personnel_account.borrow_activation_key() == activation_key, EInvalidActivationKey);
    assert!(hospital_personnel_account.borrow_is_activation_key_used(), EAccountNotActivated);
}

/// ## Params
/// - `activation_key`: argon_hash(<raw_uuid_v4>@<raw_id>)
/// - `hospital_id`: argon_hash(raw_hospital_id)
/// - `id`: argon_hash(raw_id)
/// - `private_metadata`: Base64 encoded
/// - `public_metadata`: Base64 encoded
entry fun signup(
    activation_key: String,
    address_id: &mut AddressId,
    hospital_id: String,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    personnel_id: String,
    private_metadata: String,
    public_metadata: String,
    ctx: &TxContext,
)
{
    let address_id_table = address_id.borrow_mut_table();
    assert!(!address_id_table.contains(ctx.sender()), EAddressAlreadyRegistered);

    let hospital_id = encode_hospital_id(hospital_id);
    let hospital_personnel_id = encode_hospital_personnel_id(hospital_id, personnel_id);

    address_id_table.add(ctx.sender(), hospital_personnel_id);

    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_mut_table();

    assert!(hospital_personnel_id_account_table.contains(hospital_personnel_id), EAccountNotFound);

    let hospital_personnel_account = hospital_personnel_id_account_table.borrow_mut(hospital_personnel_id);

    assert!(hospital_personnel_account.borrow_address().is_none(), EAccountAlreadyRegistered);
    require_account_activation(activation_key, hospital_personnel_account);

    let administrative_metadata = hospital_personnel_administrative_metadata_new(
        private_metadata,
        public_metadata
    );
    hospital_personnel_account.set_administrative_metadata(option::some(administrative_metadata));
    hospital_personnel_account.set_address(option::some(ctx.sender()));

    if (hospital_personnel_account.borrow_role() != hospital_personnel_role_admin()) {
        let access = hospital_personnel_access_new(
            vec_map::empty<String, HospitalPersonnelAccessData>(),
            vec_map::empty<String, HospitalPersonnelAccessData>(),
        );
        hospital_personnel_account.set_access(option::some(access));
    };

    if (hospital_personnel_account.borrow_role() == hospital_personnel_role_admin()) {
        hospital_personnel_account.set_personnels(option::some(vec_map::empty<String, HospitalPersonnelMetadata>()));
    };
}

#[test_only]
public(package) fun signup_test(
    activation_key: String,
    address_id: &mut AddressId,
    hospital_id: String,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    personnel_id: String,
    private_metadata: String,
    public_metadata: String,
    ctx: &TxContext,
)
{
    signup(
        activation_key,
        address_id,
        hospital_id,
        hospital_personnel_id_account,
        personnel_id,
        private_metadata,
        public_metadata,
        ctx
    );
}

/// ## Params
/// - `activation_key`: argon_hash(<raw_uuid_v4>@<raw_id>)
/// - `id`: argon_hash(raw_id)
entry fun update_account_activation_key(
    activation_key: String,
    address_id: &AddressId,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    personnel_id: String,
    ctx: &TxContext,
)
{
    let address_id_table = address_id.borrow_table();
    let hospital_admin_id = *address_id_table.borrow(ctx.sender());

    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_mut_table();

    let hospital_admin_account = hospital_personnel_id_account_table.borrow(hospital_admin_id);

    assert!(hospital_admin_account.borrow_role() == hospital_personnel_role_admin(), EIllegalActionInvalidRole);

    let hospital_personnel_id = encode_hospital_personnel_id(*hospital_admin_account.borrow_hospital_id(), personnel_id);

    assert!(hospital_personnel_id_account_table.contains(hospital_personnel_id), EAccountNotFound);

    let hospital_personnel_account = hospital_personnel_id_account_table.borrow_mut(hospital_personnel_id);
    hospital_personnel_account.set_activation_key(activation_key);
    hospital_personnel_account.set_is_activation_key_used(false);
}

/// ## Params
/// - `activation_key`: argon_hash(<raw_uuid_v4>@<raw_id>)
/// - `private_metadata`: Base64 encoded
/// - `public_metadata`: Base64 encoded
entry fun update_administrative_metadata(
    activation_key: String,
    address_id: &AddressId,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    private_metadata: String,
    public_metadata: String,
    ctx: &TxContext
)
{
    let address_id_table = address_id.borrow_table();
    let hospital_personnel_id = *address_id_table.borrow(ctx.sender());
    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_mut_table();
    let hospital_personnel_account = hospital_personnel_id_account_table.borrow_mut(hospital_personnel_id);

    require_account_activation(activation_key, hospital_personnel_account);

    let hospital_personnel_administrative_metadata = hospital_personnel_account.borrow_mut_administrative_metadata().borrow_mut();
    hospital_personnel_administrative_metadata.set_public_metadata(public_metadata);
    hospital_personnel_administrative_metadata.set_private_metadata(private_metadata);
    hospital_personnel_account.set_is_profile_completed(true);
}


/// ## Params
/// - `activation_key`: argon_hash(<raw_uuid_v4>@<raw_id>)
/// - `hospital_id`: argon_hash(raw_hospital_id)
/// - `personnel_id`: argon_hash(raw_id)
entry fun use_activation_key(
    activation_key: String,
    hospital_id: String,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    personnel_id: String,
)
{
    let hospital_id = encode_hospital_id(hospital_id);
    let hospital_personnel_id = encode_hospital_personnel_id(hospital_id, personnel_id);
    let hospital_personnel_id_account_table = hospital_personnel_id_account.borrow_mut_table();

    assert!(hospital_personnel_id_account_table.contains(hospital_personnel_id), EAccountNotFound);

    let hospital_personnel_account = hospital_personnel_id_account_table.borrow_mut(hospital_personnel_id);

    assert!((*hospital_personnel_account.borrow_activation_key()).into_bytes()  == activation_key.into_bytes(), EInvalidActivationKey);
    assert!(!hospital_personnel_account.borrow_is_activation_key_used(), EActivationKeyAlreadyUsed);

    hospital_personnel_account.set_is_activation_key_used(true);
}

#[test_only]
public(package) fun use_activation_key_test(
    activation_key: String,
    hospital_id: String,
    hospital_personnel_id_account: &mut HospitalPersonnelIdAccount,
    personnel_id: String,
)
{
    use_activation_key(
        activation_key,
        hospital_id,
        hospital_personnel_id_account,
        personnel_id,
    );
}
