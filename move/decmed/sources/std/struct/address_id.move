module decmed::std_struct_address_id;

use iota::table::{Self, Table};

use std::string::String;

public struct AddressId has key, store {
    id: UID,
	table: Table<address, String>,
}

public(package) fun borrow_id(
    self: &AddressId,
): &UID
{
    &self.id
}

public(package) fun borrow_table(
    self: &AddressId,
): &Table<address, String>
{
    &self.table
}

public(package) fun borrow_mut_table(
    self: &mut AddressId,
): &mut Table<address, String>
{
    &mut self.table
}


fun init(ctx: &mut TxContext) {
    let address_id = AddressId {
        id: object::new(ctx),
        table: table::new<address, String>(ctx),
    };

    transfer::share_object(address_id);
}

#[test_only]
public(package) fun default(ctx: &mut TxContext): AddressId
{
    AddressId {
        id: object::new(ctx),
        table: table::new<address, String>(ctx),
    }
}
