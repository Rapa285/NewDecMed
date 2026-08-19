module ats::audit_log;

public struct AuditLogMetadata has key {
    id: UID,

    version: vector<u8>,
    log_sequence_number: u64,
    rotation_timestamp: u64,

    ipfs_cid: vector<u8>,
    file_hash: vector<u8>,

    first_record_hash: vector<u8>,
    final_record_hash: vector<u8>,

    record_count: u64,

    prev_object_id: vector<u8>,
}

public entry fun create_audit_log(
    version: vector<u8>,
    log_sequence_number: u64,
    rotation_timestamp: u64,
    ipfs_cid: vector<u8>,
    file_hash: vector<u8>,
    first_record_hash: vector<u8>,
    final_record_hash: vector<u8>,
    record_count: u64,
    prev_object_id: vector<u8>,
    ctx: &mut TxContext,
) {
    transfer::share_object(AuditLogMetadata {
        id: object::new(ctx),
        version,
        log_sequence_number,
        rotation_timestamp,
        ipfs_cid,
        file_hash,
        first_record_hash,
        final_record_hash,
        record_count,
        prev_object_id,
    });
}