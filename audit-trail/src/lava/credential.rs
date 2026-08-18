// Credential Manager
// Menggunakan ring crate untuk Ed25519 signing/verification.
// ring tidak memiliki dependency chain yang memerlukan edition2024.

use ring::{
    rand::SystemRandom,
    signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519},
};

use crate::lava::error::{LavaError, LavaResult};

pub struct Credential {
    key_pair: Ed25519KeyPair,
    pub public_key_hex: String,
    // Simpan pkcs8 bytes untuk keperluan cloning / rotation chain
    pkcs8_bytes: Vec<u8>,
}

impl Credential {
    pub fn generate() -> LavaResult<Self> {
        let rng = SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|_| LavaError::CryptoError("gagal generate key pair".into()))?;
        let pkcs8_bytes = pkcs8.as_ref().to_vec();
        let key_pair = Ed25519KeyPair::from_pkcs8(&pkcs8_bytes)
            .map_err(|_| LavaError::CryptoError("gagal load key pair dari pkcs8".into()))?;
        let public_key_hex = hex::encode(key_pair.public_key().as_ref());
        Ok(Self { key_pair, public_key_hex, pkcs8_bytes })
    }

    /// Tandatangani data, kembalikan signature hex
    pub fn sign(&self, data: &str) -> String {
        let sig = self.key_pair.sign(data.as_bytes());
        hex::encode(sig.as_ref())
    }

    /// Verifikasi signature menggunakan public key hex
    pub fn verify(public_key_hex: &str, data: &str, signature_hex: &str) -> LavaResult<()> {
        let pk_bytes = hex::decode(public_key_hex)
            .map_err(|e| LavaError::CryptoError(format!("public key hex invalid: {e}")))?;
        let sig_bytes = hex::decode(signature_hex)
            .map_err(|e| LavaError::CryptoError(format!("signature hex invalid: {e}")))?;

        let pk = UnparsedPublicKey::new(&ED25519, pk_bytes);
        pk.verify(data.as_bytes(), &sig_bytes)
            .map_err(|_| LavaError::CryptoError("verifikasi signature gagal".into()))
    }
}

pub struct CredentialManager {
    current: Credential,
    long_term: Credential,
    entries_since_rotation: u64,
}

impl CredentialManager {
    pub fn new() -> LavaResult<Self> {
        Ok(Self {
            current: Credential::generate()?,
            long_term: Credential::generate()?,
            entries_since_rotation: 0,
        })
    }

    pub fn current_public_key(&self) -> &str {
        &self.current.public_key_hex
    }

    pub fn long_term_public_key(&self) -> &str {
        &self.long_term.public_key_hex
    }

    pub fn sign(&self, data: &str) -> String {
        self.current.sign(data)
    }

    pub fn sign_long_term(&self, data: &str) -> String {
        self.long_term.sign(data)
    }

    pub fn increment(&mut self) {
        self.entries_since_rotation += 1;
    }

    pub fn needs_rotation(&self, c: u64) -> bool {
        self.entries_since_rotation >= c
    }

    /// Rotate: generate A baru, tandatangani dengan A lama
    /// Return (new_pk_hex, signature_of_new_pk_by_old)
    pub fn rotate(&mut self) -> LavaResult<(String, String)> {
        let new_cred = Credential::generate()?;
        let signature = self.current.sign(&new_cred.public_key_hex);
        self.current = new_cred;
        self.entries_since_rotation = 0;
        Ok((self.current.public_key_hex.clone(), signature))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let cred = Credential::generate().unwrap();
        let data = "test-payload-123";
        let sig = cred.sign(data);
        assert!(Credential::verify(&cred.public_key_hex, data, &sig).is_ok());
    }

    #[test]
    fn test_verify_fails_on_tamper() {
        let cred = Credential::generate().unwrap();
        let sig = cred.sign("original");
        assert!(Credential::verify(&cred.public_key_hex, "tampered", &sig).is_err());
    }

    #[test]
    fn test_rotation_chain() {
        let mut mgr = CredentialManager::new().unwrap();
        let old_pk = mgr.current_public_key().to_string();
        let (new_pk, sig) = mgr.rotate().unwrap();
        assert!(Credential::verify(&old_pk, &new_pk, &sig).is_ok());
        assert_ne!(old_pk, new_pk);
    }
}