module ats::audit_proof {
    use std::string::String;
    use iota::object::{Self, UID};
    use iota::transfer;
    use iota::tx_context::{Self, TxContext};
    use iota::event;

    // =================================================================
    // 1. STRUCT (MODEL DATA)
    // =================================================================
    
    /// Struct ini memiliki ability `key`, artinya ia adalah Object
    /// mandiri yang akan hidup di dalam jaringan blockchain IOTA.
    public struct IntegrityProof has key {
        id: UID,                 // Wajib ada untuk object on-chain
        audit_record_id: String, // UUID v7 dari database PostgreSQL Anda
        ipfs_cid: String,        // CID dari file JSON di IPFS
        creator: address,        // Alamat dompet pembuat log
    }

    /// (Opsional tapi sangat direkomendasikan) 
    /// Event yang dipancarkan saat bukti berhasil dibuat. 
    /// Ini memudahkan PostgreSQL / Backend Anda menangkap notifikasi.
    public struct ProofCreatedEvent has copy, drop {
        proof_id: iota::object::ID,
        audit_record_id: String,
        ipfs_cid: String,
    }

    // =================================================================
    // 2. CREATOR FUNCTION
    // =================================================================

    public fun create_proof(
        audit_record_id: String,
        ipfs_cid: String,
        ctx: &mut TxContext
    ) {
        // Buat ID unik on-chain
        let id = object::new(ctx);
        let creator = tx_context::sender(ctx);

        // Pancarkan event ke luar blockchain (ditangkap oleh backend)
        event::emit(ProofCreatedEvent {
            proof_id: object::uid_to_inner(&id),
            audit_record_id,
            ipfs_cid,
        });

        // Bentuk datanya
        let proof = IntegrityProof {
            id,
            audit_record_id,
            ipfs_cid,
            creator,
        };

        // KUNCI PERMANEN (IMMUTABLE)! 
        // Object ini tidak akan pernah bisa diubah (tidak ada setter).
        transfer::freeze_object(proof);
    }

    // =================================================================
    // 3. GETTER FUNCTIONS (Bisa dibaca publik)
    // =================================================================

    public fun get_audit_record_id(proof: &IntegrityProof): &String {
        &proof.audit_record_id
    }

    public fun get_ipfs_cid(proof: &IntegrityProof): &String {
        &proof.ipfs_cid
    }

    public fun get_creator(proof: &IntegrityProof): address {
        proof.creator
    }

    //public fun vaidate_audit_record (): bool{
    //
    //}
}