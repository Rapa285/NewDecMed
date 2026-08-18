// Verification Algorithm
// Implementasi dari LAVA Algorithm 3 (paper Section 6.1).
//
// Verifier membaca file.log dari awal, memvalidasi:
//   1. Hash chain — setiap entry hash harus cocok dengan H(prev || i || data)
//   2. Authenticator — signature harus valid menggunakan current public key
//   3. Credential chain — setiap CredentialUpdate harus ditandatangani key lama
//   4. Verification anchor — ditandatangani long-term key E
//   5. Truncation — jika ada gap index yang tidak wajar
//
// Output: Ok(VerificationReport) jika log valid, Err jika ada pelanggaran.

use std::path::Path;
use tokio::{fs::File, io::{AsyncBufReadExt, BufReader}};

use crate::lava::{
    credential::Credential,
    error::{LavaError, LavaResult},
    hash_chain::{HashChain, GENESIS_HASH},
    types::{LavaParams, LogItem},
};

#[derive(Debug)]
pub struct VerificationReport {
    pub total_items: u64,
    pub total_entries: u64,
    pub total_authenticators: u64,
    pub total_credential_updates: u64,
    pub total_metronomes: u64,
    pub is_valid: bool,
}

pub struct Verifier {
    params: LavaParams,
    /// Initial public key dari IOTA — jangkar kepercayaan
    initial_public_key: String,
    /// Long-term public key E dari IOTA
    long_term_public_key: String,
}

impl Verifier {
    pub fn new(
        params: LavaParams,
        initial_public_key: String,
        long_term_public_key: String,
    ) -> Self {
        Self {
            params,
            initial_public_key,
            long_term_public_key,
        }
    }

    /// Verifikasi penuh file log dari awal hingga akhir.
    /// Sesuai Algorithm 3 di paper.
    pub async fn verify_file(&self, path: &Path) -> LavaResult<VerificationReport> {
        let file = File::open(path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut hash_chain = HashChain::from_existing(GENESIS_HASH.to_string());
        let mut current_pk = self.initial_public_key.clone();
        let mut expected_index: u64 = 0;

        let mut report = VerificationReport {
            total_items: 0,
            total_entries: 0,
            total_authenticators: 0,
            total_credential_updates: 0,
            total_metronomes: 0,
            is_valid: false,
        };

        while let Some(line) = lines.next_line().await? {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let item: LogItem = serde_json::from_str(line)
                .map_err(|e| LavaError::Serialization(e))?;

            report.total_items += 1;

            match &item {
                // ── Log entry biasa ─────────────────────────────────────────────
                LogItem::Entry(entry) => {
                    report.total_entries += 1;

                    // Cek index berurutan — gap berarti entry hilang
                    if entry.index != expected_index {
                        return Err(LavaError::VerificationFailed {
                            index: entry.index,
                            reason: format!(
                                "index tidak berurutan: expected {}, got {}",
                                expected_index, entry.index
                            ),
                        });
                    }

                    // Verifikasi hash chain
                    let data_str = serde_json::to_string(&entry.data)?;
                    let expected_hash = HashChain::compute(
                        hash_chain.current(),
                        entry.index,
                        &data_str,
                    );
                    if entry.hash != expected_hash {
                        return Err(LavaError::HashChainBroken {
                            index: entry.index,
                            expected: expected_hash,
                            got: entry.hash.clone(),
                        });
                    }

                    // Advance verifier hash chain
                    hash_chain = HashChain::from_existing(entry.hash.clone());
                    expected_index += 1;
                }

                // ── Metronome entry ─────────────────────────────────────────────
                LogItem::Metronome(metro) => {
                    report.total_metronomes += 1;

                    if metro.index != expected_index {
                        return Err(LavaError::VerificationFailed {
                            index: metro.index,
                            reason: format!(
                                "metronome index tidak berurutan: expected {}, got {}",
                                expected_index, metro.index
                            ),
                        });
                    }

                    // Metronome juga masuk hash chain
                    let metro_data = format!("metronome:{}", metro.timestamp.timestamp_millis());
                    let expected_hash = HashChain::compute(
                        hash_chain.current(),
                        metro.index,
                        &metro_data,
                    );
                    if metro.hash != expected_hash {
                        return Err(LavaError::HashChainBroken {
                            index: metro.index,
                            expected: expected_hash,
                            got: metro.hash.clone(),
                        });
                    }

                    hash_chain = HashChain::from_existing(metro.hash.clone());
                    expected_index += 1;
                }

                // ── Authenticator ───────────────────────────────────────────────
                LogItem::Authenticator(auth) => {
                    report.total_authenticators += 1;

                    // Verifikasi: signature atas hash menggunakan current_pk
                    Credential::verify(&current_pk, &auth.hash, &auth.signature)
                        .map_err(|_| LavaError::InvalidAuthenticator {
                            index: auth.entry_index,
                        })?;

                    // Hash yang di-cover authenticator harus cocok dengan chain saat ini
                    if auth.hash != hash_chain.current() {
                        return Err(LavaError::VerificationFailed {
                            index: auth.entry_index,
                            reason: "hash authenticator tidak cocok dengan chain saat ini".into(),
                        });
                    }
                }

                // ── Credential update ───────────────────────────────────────────
                LogItem::CredentialUpdate(update) => {
                    report.total_credential_updates += 1;

                    // Verifikasi: new_public_key ditandatangani key lama
                    Credential::verify(
                        &current_pk,
                        &update.new_public_key,
                        &update.signature,
                    )
                    .map_err(|_| LavaError::InvalidCredentialUpdate {
                        index: update.entry_index,
                    })?;

                    // Ganti current public key ke yang baru
                    current_pk = update.new_public_key.clone();
                }

                // ── Verification anchor ─────────────────────────────────────────
                LogItem::VerificationAnchor(anchor) => {
                    // Verifikasi signature anchor oleh long-term key E
                    let anchor_data = format!(
                        "{}:{}:{}",
                        anchor.hash, anchor.current_public_key, anchor.entry_index
                    );
                    Credential::verify(
                        &self.long_term_public_key,
                        &anchor_data,
                        &anchor.signature,
                    )
                    .map_err(|_| LavaError::VerificationFailed {
                        index: anchor.entry_index,
                        reason: "verification anchor signature tidak valid".into(),
                    })?;
                }
            }
        }

        report.is_valid = true;
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lava::{engine::LavaEngine, types::LavaParams, writer::LogWriter};
    use std::sync::Arc;
    use tempfile::NamedTempFile;
    use tokio::sync::{mpsc, Mutex};

    async fn build_log(params: LavaParams, events: Vec<serde_json::Value>) -> (
        NamedTempFile,
        String, // initial_pk
        String, // long_term_pk
    ) {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut engine = LavaEngine::new(params.clone(), tx).unwrap();
        let initial_pk = engine.initial_public_key().to_string();
        let lt_pk = engine.long_term_public_key().to_string();

        for event in events {
            engine.process_event(event).unwrap();
        }
        drop(engine); // close channel

        let tmpfile = NamedTempFile::new().unwrap();
        let mut writer = LogWriter::new(tmpfile.path().to_path_buf(), params.b);
        while let Some(item) = rx.recv().await {
            writer.push(item).await.unwrap();
        }
        writer.flush().await.unwrap();

        (tmpfile, initial_pk, lt_pk)
    }

    #[tokio::test]
    async fn test_valid_log_passes_verification() {
        let params = LavaParams {
            a: 3,
            b: 5,
            c: 10,
            d: 60,
            e: 9,
        };
        let events: Vec<_> = (0..10)
            .map(|i| serde_json::json!({ "user": "alice", "action": format!("event-{i}") }))
            .collect();

        let (tmpfile, initial_pk, lt_pk) = build_log(params.clone(), events).await;

        let verifier = Verifier::new(params, initial_pk, lt_pk);
        let report = verifier.verify_file(tmpfile.path()).await.unwrap();

        assert!(report.is_valid);
        assert_eq!(report.total_entries, 10);
    }

    #[tokio::test]
    async fn test_tampered_log_fails_verification() {
        let params = LavaParams {
            a: 5,
            b: 2,
            c: 20,
            d: 60,
            e: 25,
        };
        let events: Vec<_> = (0..5)
            .map(|i| serde_json::json!({ "action": format!("event-{i}") }))
            .collect();

        let (tmpfile, initial_pk, lt_pk) = build_log(params.clone(), events).await;

        // Baca file, ubah satu baris, tulis balik
        let content = tokio::fs::read_to_string(tmpfile.path()).await.unwrap();
        let tampered = content.replacen("event-2", "TAMPERED", 1);
        tokio::fs::write(tmpfile.path(), tampered).await.unwrap();

        let verifier = Verifier::new(params, initial_pk, lt_pk);
        let result = verifier.verify_file(tmpfile.path()).await;

        assert!(result.is_err(), "log yang dimanipulasi harus gagal verifikasi");
    }
}