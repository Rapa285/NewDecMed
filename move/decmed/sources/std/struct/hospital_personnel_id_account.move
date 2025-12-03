module decmed::std_struct_hospital_personnel_id_account;

use decmed::std_struct_hospital_personnel_account::HospitalPersonnelAccount;

use iota::table::{Self, Table};

use std::string::String;

public struct HospitalPersonnelIdAccount has key, store {
    id: UID,
	table: Table<String, HospitalPersonnelAccount>,
}

public(package) fun borrow_id(
    self: &HospitalPersonnelIdAccount,
): &UID
{
    &self.id
}

public(package) fun borrow_table(
    self: &HospitalPersonnelIdAccount,
): &Table<String, HospitalPersonnelAccount>
{
    &self.table
}

public(package) fun borrow_mut_table(
    self: &mut HospitalPersonnelIdAccount,
): &mut Table<String, HospitalPersonnelAccount>
{
    &mut self.table
}


fun init(ctx: &mut TxContext) {
    let hospital_personnel_id_account = HospitalPersonnelIdAccount{
        id: object::new(ctx),
        table: table::new<String, HospitalPersonnelAccount>(ctx),
    };

    transfer::share_object(hospital_personnel_id_account);
}

#[test_only]
public(package) fun default(ctx: &mut TxContext): HospitalPersonnelIdAccount
{
    HospitalPersonnelIdAccount{
        id: object::new(ctx),
        table: table::new<String, HospitalPersonnelAccount>(ctx),
    }
}
