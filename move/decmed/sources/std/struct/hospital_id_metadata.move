module decmed::std_struct_hospital_id_metadata;

use decmed::std_struct_hospital::Hospital;

use iota::table::{Self, Table};
use iota::table_vec::{Self, TableVec};

use std::string::String;

public struct HospitalIdMetadata has key, store {
    id: UID,
    table: Table<String, u64>,
    vec: TableVec<Hospital>,
}

public(package) fun borrow_id(
    self: &HospitalIdMetadata,
): &UID
{
    &self.id
}

public(package) fun borrow_table(
    self: &HospitalIdMetadata,
): &Table<String, u64>
{
    &self.table
}

public(package) fun borrow_mut_table(
    self: &mut HospitalIdMetadata,
): &mut Table<String, u64>
{
    &mut self.table
}

public(package) fun borrow_vec(
    self: &HospitalIdMetadata,
): &TableVec<Hospital>
{
    &self.vec
}

public(package) fun borrow_mut_vec(
    self: &mut HospitalIdMetadata,
): &mut TableVec<Hospital>
{
    &mut self.vec
}


fun init(ctx: &mut TxContext) {
    let hospital_id_metadata = HospitalIdMetadata {
        id: object::new(ctx),
        table: table::new<String, u64>(ctx),
        vec: table_vec::empty<Hospital>(ctx),
    };

    transfer::share_object(hospital_id_metadata);
}

#[test_only]
public(package) fun default(ctx: &mut TxContext): HospitalIdMetadata
{
    HospitalIdMetadata {
        id: object::new(ctx),
	    table: table::new<String, u64>(ctx),
        vec: table_vec::empty<Hospital>(ctx),
    }
}
