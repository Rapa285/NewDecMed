module ats::audit_log {
    use std::string::String;
    use std::vector;
    use iota::object::{Self, UID};
    use iota::transfer;
    use iota::tx_context::TxContext;

    public struct LogRecord has store {
        json_data: String,
    }

    // 2. Objek penampung utama yang memiliki 'key' dan menyimpan array/vector
    public struct AuditLogStore has key {
        id: UID,
        records: vector<LogRecord>,
    }

    // 3. Fungsi inisialisasi yang otomatis berjalan saat smart contract di-deploy
    fun init(ctx: &mut TxContext) {
        let store = AuditLogStore {
            id: object::new(ctx),
            records: vector::empty(), // Inisialisasi array kosong
        };
        
        transfer::share_object(store);
    }

    public entry fun create_log(store: &mut AuditLogStore, json_data: String, _ctx: &mut TxContext) {
        let new_record = LogRecord {
            json_data,
        };
        
        vector::push_back(&mut store.records, new_record);
    }

    /// Kembalikan slice LogRecord dari store.
    /// cursor: indeks awal (0-based), limit: maksimum item yang dikembalikan.
    public fun get_logs(
        store: &AuditLogStore,
        cursor: u64,
        limit: u64,
        _ctx: &TxContext,
    ): vector<LogRecord> {
        let total = vector::length(&store.records);
        let mut result = vector::empty<LogRecord>();
        let mut i = cursor;
        let end = if (cursor + limit > total) { total } else { cursor + limit };
        while (i < end) {
            vector::push_back(&mut result, *vector::borrow(&store.records, i));
            i = i + 1;
        };
        result
    }

    /// Kembalikan total jumlah record di store.
    public fun record_count(store: &AuditLogStore): u64 {
        vector::length(&store.records)
    }
}