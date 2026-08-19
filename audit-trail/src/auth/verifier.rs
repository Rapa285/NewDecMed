// Event Source Verifier
// Memverifikasi setiap SignedEvent yang masuk ke ATS sebelum
// event diteruskan ke LAVA pipeline.
//
// Langkah verifikasi (sesuai Putz et al. 2019 + Barabanov 2021):
//   1. Cek source_id terdaftar di registry
//   2. Cek nonce belum pernah dipakai (replay resistance)
//   3. Bangun canonical message dari field event
//   4. Verifikasi signature menggunakan public key source dari registry
//
// Jika semua langkah lolos → event diteruskan ke LAVA engine sebagai payload.

use std::collections::HashSet;

use ring::signature::{UnparsedPublicKey, ED25519};

use crate::auth::{
    error::{AuthError, AuthResult},
    types::SignedEvent,
    registry::SourceRegistry,
};

pub struct EventVerifier {

    used_nonces: HashSet<String>,
}

impl EventVerifier {
    pub fn new() -> Self {
        Self {
            used_nonces: HashSet::new(),
        }
    }

    /// Verifikasi SignedEvent secara penuh.
    /// Mengembalikan payload jika valid, Err jika tidak.
    pub fn verify(&mut self, event: &SignedEvent) -> AuthResult<serde_json::Value> {
        // ── 1. Cek source terdaftar ───────────────────────────────────────────
        let source_info = SourceRegistry::get(&event.source_id)?;
        let public_key_hex = source_info.public_key_hex.clone();

        // ── 2. Cek replay: nonce tidak boleh digunakan dua kali ─────────────
        if self.used_nonces.contains(&event.nonce) {
            return Err(AuthError::InvalidPayload(format!(
                "nonce '{}' sudah digunakan (replay attack)",
                event.nonce
            )));
        }

        // ── 3. Bangun canonical message ───────────────────────────────────────
        let message = event
            .canonical_message()
            .map_err(AuthError::Serialization)?;

        // ── 4. Verifikasi signature Ed25519 ───────────────────────────────────
        let pk_bytes = hex::decode(&public_key_hex).map_err(|e| {
            AuthError::CryptoError(format!("public key hex invalid: {e}"))
        })?;
        let sig_bytes = hex::decode(&event.signature).map_err(|e| {
            AuthError::CryptoError(format!("signature hex invalid: {e}"))
        })?;

        let public_key = UnparsedPublicKey::new(&ED25519, pk_bytes);
        public_key
            .verify(message.as_bytes(), &sig_bytes)
            .map_err(|_| AuthError::InvalidSignature {
                source_id: event.source_id.clone(),
            })?;

        // ── 5. Tandai nonce sudah dipakai ────────────────────────────────────
        self.used_nonces.insert(event.nonce.clone());

        // ── 6. Kembalikan payload untuk diteruskan ke LAVA ───────────────────
        Ok(event.payload.clone())
    }

}