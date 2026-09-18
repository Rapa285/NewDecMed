//! Utilitas kriptografi untuk ATS:
//! - Enkripsi AES-256-GCM
//! - ECDH key wrapping dengan public key ATS server

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use p256::{
    ecdh::EphemeralSecret,
    pkcs8::DecodePublicKey,
    PublicKey as P256PublicKey,
};
use sha2::{Digest, Sha256};

/// Hasil enkripsi AES-GCM
pub struct AesEncrypted {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,      // 12 bytes
    pub key: Vec<u8>,        // 32 bytes — perlu dienkripsi sebelum dikirim
}

/// Enkripsi plaintext dengan AES-256-GCM menggunakan key yang di-generate random
pub fn aes_encrypt(plaintext: &[u8]) -> Result<AesEncrypted, String> {
    let key = Aes256Gcm::generate_key(OsRng);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng.clone());

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| format!("AES-GCM encrypt gagal: {e}"))?;

    Ok(AesEncrypted {
        ciphertext,
        nonce: nonce.to_vec(),
        key: key.to_vec(),
    })
}

/// Enkripsi AES key dengan public key ATS server menggunakan ECDH + AES-KW
///
/// Flow:
/// 1. Generate ephemeral P-256 keypair
/// 2. ECDH dengan ATS server public key → shared secret
/// 3. KDF (SHA-256) atas shared secret → wrapping key
/// 4. AES-256-GCM enkripsi AES key dengan wrapping key
/// 5. Output: ephemeral_pubkey || nonce || encrypted_aes_key (semua base64)
pub fn ecies_encrypt_key(
    aes_key: &[u8],
    server_public_key_pem: &str,
) -> Result<String, String> {
    // 1. Parse server public key
    let server_pubkey = P256PublicKey::from_public_key_pem(server_public_key_pem)
        .map_err(|e| format!("gagal parse server public key PEM: {e}"))?;

    // 2. Generate ephemeral keypair
    let ephemeral_secret = EphemeralSecret::random(&mut OsRng);
    let ephemeral_pubkey = ephemeral_secret.public_key();

    // 3. ECDH → shared secret
    let shared_secret = ephemeral_secret.diffie_hellman(&server_pubkey);

    // 4. KDF: SHA-256 atas shared secret bytes → 32-byte wrapping key
    let mut hasher = Sha256::new();
    hasher.update(shared_secret.raw_secret_bytes());
    let wrapping_key_bytes = hasher.finalize();
    let wrapping_key = Key::<Aes256Gcm>::from_slice(&wrapping_key_bytes);

    // 5. Enkripsi AES key dengan wrapping key
    let wrap_cipher = Aes256Gcm::new(wrapping_key);
    let wrap_nonce = Aes256Gcm::generate_nonce(&mut OsRng.clone());
    let encrypted_aes_key = wrap_cipher
        .encrypt(&wrap_nonce, aes_key)
        .map_err(|e| format!("gagal enkripsi AES key: {e}"))?;

    // 6. Encode ephemeral pubkey (compressed, 33 bytes)
    let ephemeral_pubkey_bytes = ephemeral_pubkey
        .to_encoded_point(true)
        .as_bytes()
        .to_vec();

    // 7. Gabungkan: base64(ephemeral_pubkey) + "." + base64(nonce) + "." + base64(enc_key)
    let result = format!(
        "{}.{}.{}",
        STANDARD.encode(&ephemeral_pubkey_bytes),
        STANDARD.encode(wrap_nonce.as_slice()),
        STANDARD.encode(&encrypted_aes_key),
    );

    Ok(result)
}

/// Dekripsi enc_aes_key (hasil ecies_encrypt_key) dengan private key ATS server
pub fn ecies_decrypt_key(
    enc_aes_key: &str,
    server_private_key_pem: &str,
) -> Result<Vec<u8>, String> {
    use p256::{pkcs8::DecodePrivateKey, SecretKey};

    // 1. Split komponen
    let parts: Vec<&str> = enc_aes_key.split('.').collect();
    if parts.len() != 3 {
        return Err("format enc_aes_key tidak valid".to_string());
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

    // 2. Parse server private key
    let server_secret = SecretKey::from_pkcs8_pem(server_private_key_pem)
        .map_err(|e| format!("gagal parse server private key: {e}"))?;

    // 3. Parse ephemeral public key
    let ephemeral_pubkey =
        P256PublicKey::from_sec1_bytes(&ephemeral_pubkey_bytes)
            .map_err(|e| format!("gagal parse ephemeral pubkey: {e}"))?;

    // 4. ECDH
    let shared_secret = p256::ecdh::diffie_hellman(
        server_secret.to_nonzero_scalar(),
        ephemeral_pubkey.as_affine(),
    );

    // 5. KDF
    let mut hasher = Sha256::new();
    hasher.update(shared_secret.raw_secret_bytes());
    let wrapping_key_bytes = hasher.finalize();
    let wrapping_key = Key::<Aes256Gcm>::from_slice(&wrapping_key_bytes);

    // 6. Dekripsi AES key
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