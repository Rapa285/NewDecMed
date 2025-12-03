module decmed::std_struct_patient_id_account;

use decmed::std_struct_patient_account::PatientAccount;

use iota::table::{Self, Table};

use std::string::String;

public struct PatientIdAccount has key, store {
    id: UID,
	table: Table<String, PatientAccount>,
}

public(package) fun borrow_id(
    self: &PatientIdAccount,
): &UID
{
    &self.id
}

public(package) fun borrow_table(
    self: &PatientIdAccount,
): &Table<String, PatientAccount>
{
    &self.table
}

public(package) fun borrow_mut_table(
    self: &mut PatientIdAccount,
): &mut Table<String, PatientAccount>
{
    &mut self.table
}


fun init(ctx: &mut TxContext) {
    let hospital_personnel_id_account = PatientIdAccount{
        id: object::new(ctx),
        table: table::new<String, PatientAccount>(ctx),
    };

    transfer::share_object(hospital_personnel_id_account);
}

#[test_only]
public(package) fun default(ctx: &mut TxContext): PatientIdAccount
{
    PatientIdAccount{
        id: object::new(ctx),
        table: table::new<String, PatientAccount>(ctx),
    }
}
