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
    registry::SourceRegistry,
    types::SignedEvent,
};

pub struct EventVerifier {
    registry: SourceRegistry,
    /// Set nonce yang sudah dipakai — mencegah replay attack.
    /// Dalam produksi, ini perlu TTL / dibatasi memori.
    used_nonces: HashSet<String>,
}

impl EventVerifier {
    pub fn new(registry: SourceRegistry) -> Self {
        Self {
            registry,
            used_nonces: HashSet::new(),
        }
    }

    /// Verifikasi SignedEvent secara penuh.
    /// Mengembalikan payload jika valid, Err jika tidak.
    pub fn verify(&mut self, event: &SignedEvent) -> AuthResult<serde_json::Value> {
        // ── 1. Cek source terdaftar ───────────────────────────────────────────
        let source_info = self.registry.get(&event.source_id)?;
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

    pub fn registry(&self) -> &SourceRegistry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::source_client::SourceClient;

    fn make_verifier(source_id: &str) -> (EventVerifier, SourceClient) {
        let client = SourceClient::new(source_id.to_string());
        let mut registry = SourceRegistry::new();
        registry
            .register(source_id, client.public_key_hex(), None)
            .unwrap();
        (EventVerifier::new(registry), client)
    }

    #[test]
    fn test_valid_event_passes() {
        let (mut verifier, client) = make_verifier("svc-a");
        let event = client
            .sign_event(serde_json::json!({ "action": "login" }))
            .unwrap();
        assert!(verifier.verify(&event).is_ok());
    }

    #[test]
    fn test_tampered_payload_fails() {
        let (mut verifier, client) = make_verifier("svc-b");
        let mut event = client
            .sign_event(serde_json::json!({ "action": "login" }))
            .unwrap();
        // Ubah payload setelah signing
        event.payload = serde_json::json!({ "action": "ADMIN_OVERRIDE" });
        assert!(matches!(
            verifier.verify(&event),
            Err(AuthError::InvalidSignature { .. })
        ));
    }

    #[test]
    fn test_replay_attack_rejected() {
        let (mut verifier, client) = make_verifier("svc-c");
        let event = client
            .sign_event(serde_json::json!({ "action": "delete" }))
            .unwrap();
        // Pertama kali: OK
        verifier.verify(&event).unwrap();
        // Kedua kali dengan nonce yang sama: GAGAL
        let result = verifier.verify(&event);
        assert!(matches!(result, Err(AuthError::InvalidPayload(_))));
    }

    #[test]
    fn test_unknown_source_rejected() {
        let (mut verifier, _) = make_verifier("svc-d");
        let impersonator = SourceClient::new("ghost".to_string());
        let event = impersonator
            .sign_event(serde_json::json!({ "action": "hack" }))
            .unwrap();
        assert!(matches!(
            verifier.verify(&event),
            Err(AuthError::UnknownSource { .. })
        ));
    }
}