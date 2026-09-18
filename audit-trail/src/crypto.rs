//! Utilitas kriptografi sisi ATS server

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use p256::{pkcs8::DecodePrivateKey, SecretKey};
use sha2::{Digest, Sha256};

/// Dekripsi enc_aes_key dengan private key ATS server
pub fn ecies_decrypt_key(
    enc_aes_key: &str,
    server_private_key_pem: &str,
) -> Result<Vec<u8>, String> {
    use p256::PublicKey as P256PublicKey;

    let parts: Vec<&str> = enc_aes_key.split('.').collect();
    if parts.len() != 3 {
        return Err(format!(
            "format enc_aes_key tidak valid, expected 3 parts, got {}",
            parts.len()
        ));
    }

    let ephemeral_pubkey_bytes = STANDARD
        .decode(parts[0])
        .map_err(|e| format!("gagal decode ephemeral pubkey: {e}"))?;
    let wrap_nonce_bytes = STANDARD
        .decode(parts[1])
        .map_err(|e| format!("gagal decode wrap nonce: {e}"))?;
    let encrypted_aes_key = STANDARD
        .decode(parts[2])
        .map_err(|e| format!("gagal decode encrypted aes key: {e}"))?;

    // Parse server private key
    let server_secret = SecretKey::from_pkcs8_pem(server_private_key_pem)
        .map_err(|e| format!("gagal parse server private key PEM: {e}"))?;

    // Parse ephemeral public key
    let ephemeral_pubkey = P256PublicKey::from_sec1_bytes(&ephemeral_pubkey_bytes)
        .map_err(|e| format!("gagal parse ephemeral pubkey: {e}"))?;

    // ECDH
    let shared_secret = p256::ecdh::diffie_hellman(
        server_secret.to_nonzero_scalar(),
        ephemeral_pubkey.as_affine(),
    );

    // KDF: SHA-256
    let mut hasher = Sha256::new();
    hasher.update(shared_secret.raw_secret_bytes());
    let wrapping_key_bytes = hasher.finalize();
    let wrapping_key = Key::<Aes256Gcm>::from_slice(&wrapping_key_bytes);

    // Dekripsi AES key
    let wrap_cipher = Aes256Gcm::new(wrapping_key);
    let wrap_nonce = Nonce::from_slice(&wrap_nonce_bytes);
    let aes_key = wrap_cipher
        .decrypt(wrap_nonce, encrypted_aes_key.as_slice())
        .map_err(|e| format!("gagal dekripsi AES key: {e}"))?;

    Ok(aes_key)
}

/// Dekripsi ciphertext dengan AES-256-GCM
pub fn aes_decrypt(
    ciphertext: &[u8],
    key: &[u8],
    nonce: &[u8],
) -> Result<Vec<u8>, String> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("AES-GCM decrypt gagal: {e}"))
}