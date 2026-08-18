// iota_client.rs
// Wrapper untuk IOTA SDK — menyimpan metadata log ke Tangle
// sebagai Tagged Data Block yang immutable.
//
// Tag  : "ATS-AUDIT-LOG" (untuk pencarian di explorer)
// Data : JSON metadata (CID, public keys, params, dll)
//
// Block ID yang dikembalikan adalah bukti immutable bahwa
// metadata tersebut tercatat di Tangle pada waktu tertentu.

use iota_sdk::client::{Client, Result as IotaResult};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::audit_error::AuditError;

/// Tag yang dipakai untuk semua block ATS di Tangle
/// Memudahkan pencarian di IOTA explorer
const IOTA_TAG: &str = "ATS-AUDIT-LOG";

/// Metadata yang disimpan ke IOTA per file log
/// Ini adalah payload lengkap yang di-serialize ke JSON
/// lalu ditulis sebagai data di Tagged Data Block
#[derive(Debug, Serialize, Deserialize)]
pub struct IotaLogMetadata {
    /// Versi format — untuk backward compatibility
    pub version: &'static str,

    /// Nomor urut file log — untuk deteksi file yang di-skip
    pub log_sequence_number: u64,

    /// Waktu rotasi log terjadi
    pub rotation_timestamp: DateTime<Utc>,

    /// Total audit entries dalam file (bukan termasuk LAVA internal items)
    pub entry_count: u64,

    /// Parameter LAVA yang digunakan
    pub lava_params: LavaParamsMeta,

    /// Jangkar kepercayaan verifikasi — WAJIB ada di IOTA
    /// Verifier mengambil ini untuk memulai validasi hash chain
    pub initial_public_key: String,

    /// Long-term key E untuk fast-forward verification
    pub long_term_public_key: String,

    /// CID file di IPFS — diisi setelah upload berhasil
    pub ipfs_cid: String,

    /// SHA-256 hash file.log — untuk cross-check integritas
    pub file_hash: String,

    /// Daftar source yang mengirim event ke file ini
    pub source_ids: Vec<String>,

    /// Block ID IOTA dari file log sebelumnya — membentuk chain of custody
    /// None jika ini file pertama
    pub prev_iota_block_id: Option<String>,
}

/// Subset LavaParams untuk serialisasi ke IOTA
#[derive(Debug, Serialize, Deserialize)]
pub struct LavaParamsMeta {
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u64,
    pub e: u64,
}

pub struct IotaLogClient {
    node_url: String,
}

impl IotaLogClient {
    pub fn new(node_url: String) -> Self {
        Self { node_url }
    }

    /// Kirim metadata ke IOTA Tangle sebagai Tagged Data Block.
    /// Mengembalikan Block ID — simpan ini sebagai bukti immutable.
    pub async fn publish_metadata(
        &self,
        metadata: &IotaLogMetadata,
    ) -> Result<String, AuditError> {
        // Build client — koneksi ke IOTA node
        let client = Client::builder()
            .with_node(&self.node_url)
            .map_err(|e| AuditError::from(anyhow::anyhow!("IOTA client error: {e}")))?
            .finish()
            .await
            .map_err(|e| AuditError::from(anyhow::anyhow!("IOTA client finish error: {e}")))?;

        // Serialize metadata ke JSON bytes
        let data = serde_json::to_string(metadata)
            .map_err(|e| AuditError::from(anyhow::anyhow!("Serialisasi metadata IOTA gagal: {e}")))?;

        // Kirim sebagai Tagged Data Block
        // Tag  : untuk pencarian di explorer
        // Data : JSON metadata
        let block = client
            .build_block()
            .with_tag(IOTA_TAG.as_bytes().to_vec())
            .with_data(data.as_bytes().to_vec())
            .finish()
            .await
            .map_err(|e| AuditError::from(anyhow::anyhow!("Gagal kirim block ke IOTA: {e}")))?;

        let block_id = block.id().to_string();
        println!("[iota] metadata tersimpan di Tangle. Block ID: {block_id}");

        Ok(block_id)
    }

    /// Hitung SHA-256 hash dari file — untuk field file_hash di metadata
    pub async fn hash_file(file_path: &str) -> Result<String, AuditError> {
        use sha2::{Digest, Sha256};
        use tokio::io::AsyncReadExt;

        let mut file = tokio::fs::File::open(file_path)
            .await
            .map_err(|e| AuditError::from(anyhow::anyhow!("Gagal buka file untuk hash: {e}")))?;

        let mut hasher = Sha256::new();
        let mut buf = [0u8; 8192];

        loop {
            let n = file.read(&mut buf).await
                .map_err(|e| AuditError::from(anyhow::anyhow!("Gagal baca file: {e}")))?;
            if n == 0 { break; }
            hasher.update(&buf[..n]);
        }

        Ok(hex::encode(hasher.finalize()))
    }
}