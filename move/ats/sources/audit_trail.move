module ats::audit_log {
    use std::string::String;
    use iota::object::{Self, UID};
    use iota::transfer;
    use iota::tx_context::TxContext;

    public struct LogRecord has key {
        id: UID,
        json_data: String,
    }

    public entry fun create_log(json_data: String, ctx: &mut TxContext) {
        // 1. New LogRecord
        let record = LogRecord {
            id: object::new(ctx),
            json_data,
        };
        
        // 2. Freeze object
        transfer::freeze_object(record);
    }
}