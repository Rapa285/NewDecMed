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

pub struct SourceRegistry {
    sources: HashMap<String, SourceInfo>,
}

impl SourceRegistry {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    /// Daftarkan source baru. Dipanggil saat ATS startup dari konfigurasi.
    /// Menolak duplikasi source_id.
    pub fn register(
        &mut self,
        source_id: impl Into<String>,
        public_key_hex: impl Into<String>,
        description: Option<String>,
    ) -> AuthResult<()> {
        let source_id = source_id.into();
        if self.sources.contains_key(&source_id) {
            return Err(AuthError::DuplicateSource {
                source_id,
            });
        }
        self.sources.insert(
            source_id.clone(),
            SourceInfo {
                source_id,
                public_key_hex: public_key_hex.into(),
                description,
                registered_at: Utc::now(),
            },
        );
        Ok(())
    }

    /// Ambil info source. Err jika tidak terdaftar.
    pub fn get(&self, source_id: &str) -> AuthResult<&SourceInfo> {
        self.sources.get(source_id).ok_or_else(|| AuthError::UnknownSource {
            source_id: source_id.to_string(),
        })
    }

    pub fn len(&self) -> usize {
        self.sources.len()
    }

    /// Kembalikan semua source terdaftar — untuk metadata IOTA
    pub fn all(&self) -> Vec<&SourceInfo> {
        self.sources.values().collect()
    }
}

impl Default for SourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_get() {
        let mut reg = SourceRegistry::new();
        reg.register("svc-a", "deadbeef", Some("Service A".into())).unwrap();
        let info = reg.get("svc-a").unwrap();
        assert_eq!(info.source_id, "svc-a");
        assert_eq!(info.public_key_hex, "deadbeef");
    }

    #[test]
    fn test_duplicate_rejected() {
        let mut reg = SourceRegistry::new();
        reg.register("svc-a", "pk1", None).unwrap();
        let res = reg.register("svc-a", "pk2", None);
        assert!(matches!(res, Err(AuthError::DuplicateSource { .. })));
    }

    #[test]
    fn test_unknown_source() {
        let reg = SourceRegistry::new();
        let res = reg.get("ghost");
        assert!(matches!(res, Err(AuthError::UnknownSource { .. })));
    }
}