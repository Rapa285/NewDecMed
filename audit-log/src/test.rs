// audit-trail/src/tests.rs
//
// Jalankan dengan:
//   cargo test                          — semua test
//   cargo test verify_record            — P-KNF-01
//   cargo test verify_chain             — P-KNF-02 + P-INT-01
//   cargo test format_roundtrip         — P-KNF-03
//   cargo test signed_event             — P-KNF-06, 07, 08
//
// Tambahkan baris berikut di audit-trail/src/main.rs (atau lib.rs):
//   #[cfg(test)] mod tests;

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;
    use uuid::Uuid;

    use crate::{
        audit::create_audit_record,
        types::{
            AuditActionType, AuditActorType, AuditEvent, AuditEventDetails,
            AuditOutcome, AuditRecord, AuditSourceComponent, AuditTargetObjectType,
            Event, SignedEvent,
        },
        utils::Utils,
    };

    // ── Helper: buat AuditEvent EV4 (Medical Record Access) ──────────────────

    fn make_ev4_event() -> AuditEvent {
        AuditEvent {
            source_component: AuditSourceComponent::HospitalClient,
            source_timestamp: Utc::now(),
            event: Event {
                actor_id: "0xabc123_tenaga_medis".to_string(),
                actor_type: AuditActorType::MedicalPersonnel,
                target_object_type: AuditTargetObjectType::MedicalRecord,
                target_object: "ipfs://bafybeig_cid_rme".to_string(),
                outcome: AuditOutcome::Success,
                action_type: AuditActionType::Read,
                details: AuditEventDetails::MedicalRecordAccess {
                    patient_iota_address: "0xpatient_iota_addr".to_string(),
                    record_index: Some(0),
                    role_used: "MedicalPersonnel".to_string(),
                },
            },
        }
    }

    /// Helper: buat AuditEvent sederhana EV1 (Authentication).
    fn make_ev1_event(actor: &str, outcome: AuditOutcome) -> AuditEvent {
        AuditEvent {
            source_component: AuditSourceComponent::HospitalClient,
            source_timestamp: Utc::now(),
            event: Event {
                actor_id: actor.to_string(),
                actor_type: AuditActorType::MedicalPersonnel,
                target_object_type: AuditTargetObjectType::AccessKeys,
                target_object: "session".to_string(),
                outcome,
                action_type: AuditActionType::SignIn,
                details: AuditEventDetails::Authentication {
                    auth_method: "iota_signature".to_string(),
                    role: "MedicalPersonnel".to_string(),
                    failure_reason: None,
                },
            },
        }
    }

    // ── Buat rantai N record secara berurutan ─────────────────────────────────

    fn build_chain(n: usize) -> Vec<AuditRecord> {
        let mut records = Vec::new();
        let mut prev_hash: Option<String> = None;

        for i in 0..n {
            let event = make_ev1_event(&format!("actor_{i}"), AuditOutcome::Success);
            let record = create_audit_record(event, prev_hash.clone());
            prev_hash = Some(record.record_hash.clone());
            records.push(record);
        }

        records
    }

    // ── Buat SignedEvent valid menggunakan ed25519-dalek ──────────────────────

    fn make_signed_event(event: &AuditEvent) -> (SignedEvent, SigningKey) {
        let signing_key = SigningKey::generate(&mut OsRng);
        let payload = serde_json::to_string(event).expect("serialize event");
        let sig = signing_key.sign(payload.as_bytes());

        let signed = SignedEvent {
            payload,
            signature: hex::encode(sig.to_bytes()),
            public_key: hex::encode(signing_key.verifying_key().as_bytes()),
        };

        (signed, signing_key)
    }

    // =========================================================================
    // P-KNF-01: verify_record mendeteksi modifikasi pada satu record
    // =========================================================================

    #[test]
    fn verify_record_valid() {
        let event = make_ev4_event();
        let record = create_audit_record(event, None);
        assert!(
            Utils::verify_record(&record),
            "Record yang baru dibuat harus valid"
        );
    }

    #[test]
    fn verify_record_tampered_actor_id() {
        let event = make_ev4_event();
        let mut record = create_audit_record(event, None);

        // Modifikasi langsung: ubah actor_id
        record.event.event.actor_id = "0xpenyerang_palsu".to_string();

        assert!(
            !Utils::verify_record(&record),
            "Record yang dimodifikasi harus gagal verifikasi"
        );
    }

    #[test]
    fn verify_record_tampered_outcome() {
        let event = make_ev4_event();
        let mut record = create_audit_record(event, None);

        // Modifikasi: ubah outcome dari Success menjadi Failure
        record.event.event.outcome = AuditOutcome::Failure;

        assert!(
            !Utils::verify_record(&record),
            "Perubahan outcome harus terdeteksi"
        );
    }

    // =========================================================================
    // P-KNF-02: verify_chain mendeteksi perusakan di posisi tengah rantai
    // =========================================================================

    #[test]
    fn verify_chain_three_records_valid() {
        let records = build_chain(3);
        assert!(
            Utils::verify_chain(&records).is_ok(),
            "Rantai tiga record yang tidak dimodifikasi harus valid"
        );
    }

    #[test]
    fn verify_chain_tamper_middle_record() {
        let mut records = build_chain(3);

        // Modifikasi record ke-2 (indeks 1): ubah actor_id
        records[1].event.event.actor_id = "0xpenyerang".to_string();

        let result = Utils::verify_chain(&records);
        assert!(result.is_err(), "Perusakan pada record ke-2 harus terdeteksi");

        let msg = result.unwrap_err();
        // Record ke-2 gagal karena hash mismatch
        assert!(
            msg.contains("record 2"),
            "Pesan error harus menyebut record 2, dapat: {msg}"
        );
    }

    #[test]
    fn verify_chain_tamper_first_record() {
        let mut records = build_chain(3);

        // Modifikasi record pertama
        records[0].event.event.actor_id = "0xpenyerang".to_string();

        let result = Utils::verify_chain(&records);
        assert!(result.is_err(), "Perusakan pada record ke-1 harus terdeteksi");

        let msg = result.unwrap_err();
        // Record ke-1 gagal hash mismatch; record ke-2 kemudian gagal prev_hash mismatch
        assert!(
            msg.contains("record 1") || msg.contains("record 2"),
            "Efek domino harus terdeteksi: {msg}"
        );
    }

    #[test]
    fn verify_chain_single_record_valid() {
        let records = build_chain(1);
        assert!(Utils::verify_chain(&records).is_ok());
    }

    #[test]
    fn verify_chain_empty() {
        let records: Vec<AuditRecord> = vec![];
        // Rantai kosong dianggap valid (tidak ada yang perlu diverifikasi)
        assert!(Utils::verify_chain(&records).is_ok());
    }

    // =========================================================================
    // P-KNF-03: format JSON-Lines dapat di-deserialisasi kembali ke AuditRecord
    // =========================================================================

    #[test]
    fn format_roundtrip_single_record() {
        let event = make_ev4_event();
        let record = create_audit_record(event, None);

        // Serialisasi ke JSON (simulasi satu baris di berkas log)
        let json_line = serde_json::to_string(&record).expect("serialize record");

        // Deserialisasi kembali
        let decoded: AuditRecord =
            serde_json::from_str(&json_line).expect("deserialize record");

        assert_eq!(record.record_id, decoded.record_id);
        assert_eq!(record.record_hash, decoded.record_hash);
        assert_eq!(
            record.event.event.actor_id,
            decoded.event.event.actor_id
        );
    }

    #[test]
    fn format_roundtrip_chain_five_records() {
        let records = build_chain(5);

        // Simulasi tulis ke berkas log (JSON-Lines)
        let jsonl: String = records
            .iter()
            .map(|r| serde_json::to_string(r).unwrap())
            .collect::<Vec<_>>()
            .join("\n");

        // Baca kembali baris per baris
        let decoded: Vec<AuditRecord> = jsonl
            .lines()
            .map(|line| serde_json::from_str(line).expect("tiap baris valid JSON"))
            .collect();

        assert_eq!(records.len(), decoded.len(), "Jumlah record harus sama");

        for (original, parsed) in records.iter().zip(decoded.iter()) {
            assert_eq!(original.record_id, parsed.record_id);
            assert_eq!(original.record_hash, parsed.record_hash);
            assert_eq!(original.prev_record_hash, parsed.prev_record_hash);
        }

        // Rantai yang dibaca kembali harus tetap valid
        assert!(Utils::verify_chain(&decoded).is_ok());
    }

    // =========================================================================
    // P-KNF-06: verify_and_extract_event berhasil pada SignedEvent yang valid
    // =========================================================================

    #[test]
    fn signed_event_valid_accepted() {
        let event = make_ev1_event("0xactor_sah", AuditOutcome::Success);
        let (signed, _key) = make_signed_event(&event);

        let result = Utils::verify_and_extract_event(&signed);
        assert!(
            result.is_ok(),
            "Event dengan tanda tangan valid harus diterima: {:?}",
            result.err()
        );

        let extracted = result.unwrap();
        assert_eq!(extracted.event.actor_id, "0xactor_sah");
    }

    // =========================================================================
    // P-KNF-07: payload yang dimodifikasi setelah penandatanganan ditolak
    // =========================================================================

    #[test]
    fn signed_event_tampered_payload_rejected() {
        let event = make_ev1_event("0xactor_sah", AuditOutcome::Success);
        let (mut signed, _key) = make_signed_event(&event);

        // Modifikasi satu karakter pada payload setelah penandatanganan
        let original_len = signed.payload.len();
        signed.payload.push_str("x"); // tambah karakter tidak sah
        assert_ne!(signed.payload.len(), original_len);

        let result = Utils::verify_and_extract_event(&signed);
        assert!(
            result.is_err(),
            "Payload yang dimodifikasi harus ditolak"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.to_lowercase().contains("signature")
                || err.to_lowercase().contains("tanda tangan")
                || err.to_lowercase().contains("invalid"),
            "Pesan error harus mengindikasikan kegagalan verifikasi: {err}"
        );
    }

    // =========================================================================
    // P-KNF-08: tanda tangan dari kunci yang berbeda ditolak
    // =========================================================================

    #[test]
    fn signed_event_wrong_key_rejected() {
        let event = make_ev1_event("0xactor_sah", AuditOutcome::Success);
        let (mut signed, _original_key) = make_signed_event(&event);

        // Ganti signature dengan tanda tangan dari keypair berbeda
        let different_key = SigningKey::generate(&mut OsRng);
        let wrong_sig = different_key.sign(signed.payload.as_bytes());
        signed.signature = hex::encode(wrong_sig.to_bytes());
        // public_key tetap milik original_key → mismatch

        let result = Utils::verify_and_extract_event(&signed);
        assert!(
            result.is_err(),
            "Tanda tangan dari kunci berbeda harus ditolak"
        );
    }

    #[test]
    fn signed_event_wrong_public_key_rejected() {
        let event = make_ev1_event("0xactor_sah", AuditOutcome::Success);
        let (mut signed, _key) = make_signed_event(&event);

        // Ganti public_key dengan kunci publik acak yang tidak bersesuaian
        let impostor_key = SigningKey::generate(&mut OsRng);
        signed.public_key = hex::encode(impostor_key.verifying_key().as_bytes());

        let result = Utils::verify_and_extract_event(&signed);
        assert!(
            result.is_err(),
            "Public key yang tidak bersesuaian harus ditolak"
        );
    }

    // =========================================================================
    // P-INT-01: siklus lengkap — EV4, record tersimpan, rantai valid,
    //           perusakan terdeteksi (hash-chain + efek domino)
    // =========================================================================

    #[test]
    fn full_cycle_ev4_chain_valid() {
        // Langkah 1: Bangkitkan event EV4
        let ev4 = make_ev4_event();

        // Langkah 2: Buat tiga record berurutan; record kedua adalah EV4
        let rec1 = create_audit_record(make_ev1_event("actor_1", AuditOutcome::Success), None);
        let rec2 = create_audit_record(ev4, Some(rec1.record_hash.clone()));
        let rec3 = create_audit_record(
            make_ev1_event("actor_3", AuditOutcome::Success),
            Some(rec2.record_hash.clone()),
        );

        // Langkah 3: Verifikasi hubungan antar-record
        assert_eq!(
            rec2.prev_record_hash,
            Some(rec1.record_hash.clone()),
            "prev_record_hash record ke-2 harus = record_hash record ke-1"
        );
        assert_eq!(
            rec3.prev_record_hash,
            Some(rec2.record_hash.clone()),
            "prev_record_hash record ke-3 harus = record_hash record ke-2"
        );

        // Langkah 4a: Rantai valid sebelum modifikasi
        let chain = vec![rec1.clone(), rec2.clone(), rec3.clone()];
        assert!(
            Utils::verify_chain(&chain).is_ok(),
            "Rantai tiga record harus valid sebelum modifikasi"
        );

        // Langkah 4b: Rusak record ke-2, periksa efek domino
        let mut tampered_chain = chain.clone();
        tampered_chain[1].event.event.actor_id = "0xpenyerang_palsu".to_string();

        let result = Utils::verify_chain(&tampered_chain);
        assert!(
            result.is_err(),
            "Perusakan pada record ke-2 harus terdeteksi"
        );

        // Langkah 5: Dari record EV4, minimal bukti dapat disimpulkan
        let ev4_record = &chain[1];
        assert_eq!(
            ev4_record.event.event.actor_id,
            "0xabc123_tenaga_medis",
            "actor_id harus tercatat dengan tepat"
        );
        assert!(
            matches!(
                ev4_record.event.event.outcome,
                crate::types::AuditOutcome::Success
            ),
            "Outcome harus tercatat"
        );

        // Format JSON-Lines round-trip tetap valid
        let jsonl = chain
            .iter()
            .map(|r| serde_json::to_string(r).unwrap())
            .collect::<Vec<_>>()
            .join("\n");

        let decoded: Vec<AuditRecord> = jsonl
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();

        assert!(Utils::verify_chain(&decoded).is_ok());
    }

    // ── Sanity: record pertama selalu punya prev_record_hash = None ───────────

    #[test]
    fn first_record_has_null_prev_hash() {
        let event = make_ev4_event();
        let record = create_audit_record(event, None);
        assert_eq!(record.prev_record_hash, None);
    }

    // ── Sanity: record_hash berupa string hex SHA-256 (64 karakter) ───────────

    #[test]
    fn record_hash_is_64_char_hex() {
        let event = make_ev4_event();
        let record = create_audit_record(event, None);
        assert_eq!(
            record.record_hash.len(),
            64,
            "SHA-256 hex harus 64 karakter"
        );
        assert!(
            record.record_hash.chars().all(|c| c.is_ascii_hexdigit()),
            "record_hash harus berupa karakter hex"
        );
    }
}