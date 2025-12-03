module decmed::shared;

use iota::hash::blake2b256;
use iota::hex;

use std::string::{Self, String};

// Errors

// Structs

public struct GlobalAdminCap has key {
    id: UID,
}

public struct ProxyCap has key {
    id: UID,
}

// Functions

public(package) fun transfer_proxy_cap(
    proxy_address: address,
    ctx: &mut TxContext
)
{
    let proxy_cap = ProxyCap { id: object::new(ctx) };
    transfer::transfer(proxy_cap, proxy_address);
}

public(package) fun encode_hospital_id(
    hospital_id: String
): String
{
    string::utf8(hex::encode(blake2b256(hospital_id.as_bytes())))
}

/// ## Params
/// - `hospital_id`: argon_hash(raw_hospital_id)
/// - `id`: argon_hash(raw_id)
public(package) fun encode_hospital_personnel_id(
    hospital_id: String,
    id: String,
): String
{
    let mut hospital_personnel_id = string::utf8(b"");

    string::append(&mut hospital_personnel_id, id);
    string::append_utf8(&mut hospital_personnel_id, b"@");
    string::append(&mut hospital_personnel_id, hospital_id);

    string::utf8(hex::encode(blake2b256(hospital_personnel_id.as_bytes())))
}

fun init(ctx: &mut TxContext)
{
    transfer::transfer(GlobalAdminCap{
        id: object::new(ctx)
    }, ctx.sender());
}


public(package) fun encode_patient_id(
    patient_id: String
): String
{
    string::utf8(hex::encode(blake2b256(patient_id.as_bytes())))
}

#[test_only]
public(package) fun transfer_global_admin_cap(to: address, ctx: &mut TxContext)
{
    transfer::transfer(GlobalAdminCap{
        id: object::new(ctx)
    }, to);
}
