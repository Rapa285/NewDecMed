// iota_client.rs
// Menyimpan metadata log ke IOTA Rebased menggunakan
// IOTA Notarization Toolkit (Single Notarization — Locked method).
//
// Setiap file log rotation menghasilkan satu Notarized Object on-chain:
//   - Immutable setelah dibuat (Locked Notarization)
//   - Verifiable oleh siapapun via Object ID
//   - Berisi: CID IPFS, public keys LAVA, params, file hash, dll
//
// Referensi: https://docs.iota.org/developer/iota-notarization

use notarization::{
    NotarizationClient,
    NotarizationClientReadOnly,
    builder::LockedNotarizationBuilder,
    types::TimeLock,
};
use iota_sdk::{
    IotaClientBuilder,
    types::crypto::IotaKeyPair,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

use crate::audit_error::AuditError;

// ── Metadata struct ───────────────────────────────────────────────────────────

/// Metadata yang disimpan sebagai Notarized Object di IOTA per file log.
/// Ini adalah "jangkar kepercayaan" yang mengikat:
///   - file.log di IPFS  (via ipfs_cid)
///   - verifikasi LAVA   (via initial_public_key + lt_public_key)
///   - parameter LAVA    (via lava_params)
///   - urutan file       (via log_sequence_number + prev_object_id)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IotaLogMetadata {
    pub version: String,
    pub log_sequence_number: u64,
    pub rotation_timestamp: DateTime<Utc>,
    pub lava_params: LavaParamsMeta,
    pub initial_public_key: String,
    pub long_term_public_key: String,
    pub ipfs_cid: String,
    pub file_hash: String,
    pub source_ids: Vec<String>,
    /// Object ID dari notarized object file log sebelumnya.
    /// Membentuk chain of custody on-chain.
    pub prev_object_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LavaParamsMeta {
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u64,
    pub e: u64,
}

// ── Hasil publish ─────────────────────────────────────────────────────────────

pub struct PublishResult {
    /// Object ID on-chain — ini yang disimpan sebagai referensi
    /// dan digunakan verifier untuk fetch metadata dari IOTA
    pub object_id: String,
    /// Transaction digest — bukti tambahan transaksi berhasil
    pub tx_digest: String,
}

// ── IotaLogClient ─────────────────────────────────────────────────────────────

pub struct IotaLogClient {
    node_url: String,
    /// Private key ATS untuk signing transaksi ke IOTA
    /// Dalam produksi: load dari env var / secret manager, bukan hardcode
    key_pair: IotaKeyPair,
}

impl IotaLogClient {
    /// Buat client baru.
    /// `key_pair_bech32` adalah private key dalam format bech32
    /// (format default IOTA CLI: `iota keytool export`)
    pub fn new(node_url: String, key_pair_bech32: String) -> Result<Self, AuditError> {
        let key_pair = IotaKeyPair::decode(&key_pair_bech32)
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal decode IOTA key pair: {e}")
            ))?;
        Ok(Self { node_url, key_pair })
    }

    /// Publish metadata sebagai Locked Notarization ke IOTA Rebased.
    ///
    /// Locked Notarization dipilih karena:
    ///   - Data immutable setelah dibuat — tidak bisa diubah siapapun
    ///   - Delete lock = jauh di masa depan (tahun 2099) → praktis permanent
    ///   - Verifiable oleh siapapun hanya dengan Object ID
    pub async fn publish_metadata(
        &self,
        metadata: &IotaLogMetadata,
    ) -> Result<PublishResult, AuditError> {
        // ── 1. Serialize metadata ke JSON string ──────────────────────────
        let metadata_json = serde_json::to_string(metadata)
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal serialize metadata: {e}")
            ))?;

        // ── 2. Bangun IOTA client ─────────────────────────────────────────
        let iota_client = IotaClientBuilder::default()
            .build(&self.node_url)
            .await
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal connect ke IOTA node: {e}")
            ))?;

        // ── 3. Bangun Notarization client dengan signer ───────────────────
        let read_only = NotarizationClientReadOnly::new(iota_client)
            .await
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal buat read-only client: {e}")
            ))?;

        let notarization_client = NotarizationClient::new(read_only, &self.key_pair)
            .await
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal buat notarization client: {e}")
            ))?;

        // ── 4. Bangun Locked Notarization ─────────────────────────────────
        // Delete lock = Unix timestamp tahun 2099 → praktis permanent
        // Ini menjamin metadata tidak bisa dihapus selama sistem berjalan
        let delete_lock_until: u32 = 4102444800; // 2099-12-31 UTC

        let result = notarization_client
            .create_locked()                          // LockedNotarizationBuilder
            .with_state(metadata_json.as_bytes())     // data yang dinotarisasi
            .with_description(format!(               // deskripsi human-readable
                "ATS Log #{} — {}",
                metadata.log_sequence_number,
                metadata.rotation_timestamp.format("%Y-%m-%dT%H:%M:%SZ")
            ))
            .with_delete_lock(TimeLock::UnlockAt(delete_lock_until))
            .finish()                                 // → TransactionBuilder
            .build_and_execute()                      // sign + submit ke network
            .await
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal publish ke IOTA: {e}")
            ))?;

        // ── 5. Ekstrak Object ID dari response ────────────────────────────
        let object_id = result
            .notarization_object_id()
            .map(|id| id.to_string())
            .ok_or_else(|| AuditError::from(
                anyhow::anyhow!("IOTA response tidak mengandung object ID")
            ))?;

        let tx_digest = result
            .iota_response()
            .digest
            .to_string();

        println!(
            "[iota] metadata tersimpan. Object ID: {object_id} | TX: {tx_digest}"
        );

        Ok(PublishResult { object_id, tx_digest })
    }

    /// Verifikasi metadata — fetch notarized object dari IOTA via Object ID
    /// dan bandingkan dengan metadata yang diberikan.
    /// Dipanggil oleh verifier saat audit.
    pub async fn verify_metadata(
        &self,
        object_id: &str,
        expected: &IotaLogMetadata,
    ) -> Result<bool, AuditError> {
        let iota_client = IotaClientBuilder::default()
            .build(&self.node_url)
            .await
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal connect ke IOTA node: {e}")
            ))?;

        let read_only = NotarizationClientReadOnly::new(iota_client)
            .await
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal buat read-only client: {e}")
            ))?;

        // Fetch notarized object tanpa perlu signing (read-only)
        let handle = read_only
            .notarization(object_id)
            .await
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal fetch object dari IOTA: {e}")
            ))?;

        let on_chain_state = handle
            .state()
            .await
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal baca state dari object: {e}")
            ))?;

        // Parse on-chain state kembali ke IotaLogMetadata
        let on_chain_metadata: IotaLogMetadata = serde_json::from_slice(
            on_chain_state.data()
        ).map_err(|e| AuditError::from(
            anyhow::anyhow!("gagal parse metadata dari IOTA: {e}")
        ))?;

        // Bandingkan field kritis
        let valid = on_chain_metadata.ipfs_cid == expected.ipfs_cid
            && on_chain_metadata.file_hash == expected.file_hash
            && on_chain_metadata.initial_public_key == expected.initial_public_key
            && on_chain_metadata.log_sequence_number == expected.log_sequence_number;

        Ok(valid)
    }

    /// Hitung SHA-256 hash dari file — untuk field `file_hash` di metadata
    pub async fn hash_file(file_path: &str) -> Result<String, AuditError> {
        use tokio::io::AsyncReadExt;

        let mut file = tokio::fs::File::open(file_path)
            .await
            .map_err(|e| AuditError::from(
                anyhow::anyhow!("gagal buka file untuk hash: {e}")
            ))?;

        let mut hasher = Sha256::new();
        let mut buf = [0u8; 8192];

        loop {
            let n = file.read(&mut buf).await
                .map_err(|e| AuditError::from(
                    anyhow::anyhow!("gagal baca file: {e}")
                ))?;
            if n == 0 { break; }
            hasher.update(&buf[..n]);
        }

        Ok(hex::encode(hasher.finalize()))
    }
}