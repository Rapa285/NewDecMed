// Source Registry
// Menyimpan daftar event source yang diizinkan beserta public key-nya.
// Dalam sistem nyata, registry ini di-seed saat startup dari config file
// atau secret manager. Tidak ada registrasi dinamis di runtime untuk
// menjaga attack surface tetap kecil.

use std::collections::HashMap;
use chrono::Utc;

use crate::auth::{
    error::{AuthError, AuthResult},
    types::SourceInfo,
};
use std::sync::OnceLock;

pub fn global_registry() -> &'static HashMap<String, SourceInfo> {
    static REGISTRY: OnceLock<HashMap<String, SourceInfo>> = OnceLock::new();
    
    REGISTRY.get_or_init(|| {
        let mut sources = HashMap::new();
        
        // ── HARDCODE SOURCE KAMU DI SINI ─────────────────────────────
        sources.insert(
            "TauriBackend_1".to_string(),
            SourceInfo {
                source_id: "TauriBackend_1".to_string(),
                // Ganti dengan public key Ed25519 hex yang asli nanti
                public_key_hex: "1234567890abcdef".to_string(), 
                description: Some("Tauri Backend Application".to_string()),
                registered_at: Utc::now(),
            }
        );

        // Tambahkan source lain jika ada
        // sources.insert("MobileApp_1".to_string(), SourceInfo { ... });

        sources
    })
}

pub struct SourceRegistry;

impl SourceRegistry {
    /// Ambil info source dari registry konstan. Err jika tidak terdaftar.
    pub fn get(source_id: &str) -> AuthResult<&'static SourceInfo> {
        let registry = global_registry();
        registry.get(source_id).ok_or_else(|| AuthError::UnknownSource {
            source_id: source_id.to_string(),
        })
    }

    pub fn len() -> usize {
        global_registry().len()
    }

    /// Kembalikan semua source terdaftar — untuk metadata IOTA
    pub fn all() -> Vec<&'static SourceInfo> {
        global_registry().values().collect()
    }
}
