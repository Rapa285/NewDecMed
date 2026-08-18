// mTLS Acceptor
// Mengimplementasikan mutual TLS untuk koneksi antara event source dan ATS.
//
// "Mutual" artinya kedua pihak harus membuktikan identitasnya:
//   - Server (ATS) membuktikan identitas ke client via server certificate
//   - Client (event source) membuktikan identitas ke server via client certificate
//
// Ini adalah lapisan keamanan transport di atas payload signing:
//   - mTLS  → "koneksi ini benar-benar dari mesin yang sah" (transport layer)
//   - Payload signing → "event ini benar-benar dibuat oleh source yang sah" (app layer)
//
// Implementasi menggunakan rustls 0.21 + tokio-rustls 0.24.
//
// Catatan: Untuk menyederhanakan demo, helper di sini menggunakan
// self-signed certificates yang di-generate in-memory menggunakan rcgen.
// Dalam produksi: gunakan cert dari CA internal (misalnya Vault PKI).

use std::sync::Arc;

use rustls::{
    server::AllowAnyAuthenticatedClient, Certificate, PrivateKey, RootCertStore,
    ServerConfig,
};
use tokio_rustls::TlsAcceptor;

use crate::auth::error::{AuthError, AuthResult};

/// Konfigurasi untuk mTLS server (ATS)
pub struct MtlsConfig {
    /// Server certificate chain (PEM)
    pub server_cert_pem: Vec<u8>,
    /// Server private key (PEM)
    pub server_key_pem: Vec<u8>,
    /// CA certificate yang digunakan untuk memvalidasi client cert (PEM)
    pub client_ca_cert_pem: Vec<u8>,
}

/// Bangun TlsAcceptor dari MtlsConfig.
/// Acceptor ini yang di-wrap ke setiap koneksi TCP masuk.
pub fn build_tls_acceptor(config: &MtlsConfig) -> AuthResult<TlsAcceptor> {
    // ── Load server certificate ───────────────────────────────────────────────
    let server_certs = load_certs(&config.server_cert_pem)?;
    let server_key = load_key(&config.server_key_pem)?;

    // ── Setup CA untuk verifikasi client certificate ──────────────────────────
    let mut root_store = RootCertStore::empty();
    let ca_certs = load_certs(&config.client_ca_cert_pem)?;
    for cert in ca_certs {
        root_store.add(&cert).map_err(|e| {
            AuthError::CryptoError(format!("gagal load CA cert: {e}"))
        })?;
    }

    // AllowAnyAuthenticatedClient: tolak koneksi tanpa client cert yang valid
    let client_verifier = AllowAnyAuthenticatedClient::new(root_store);

    // ── Build ServerConfig dengan client verification wajib ──────────────────
    let tls_config = ServerConfig::builder()
        .with_safe_defaults()
        .with_client_cert_verifier(Arc::new(client_verifier))
        .with_single_cert(server_certs, server_key)
        .map_err(|e| AuthError::CryptoError(format!("TLS config error: {e}")))?;

    Ok(TlsAcceptor::from(Arc::new(tls_config)))
}

/// Ekstrak source_id dari client certificate yang sudah terautentikasi.
/// Dalam implementasi ini, source_id diambil dari Common Name (CN) cert.
/// Konvensi: CN = "ats-client-{source_id}"
pub fn extract_source_id_from_cert(common_name: &str) -> Option<String> {
    common_name
        .strip_prefix("ats-client-")
        .map(|s| s.to_string())
}

// ── PEM parsing helpers ───────────────────────────────────────────────────────

fn load_certs(pem: &[u8]) -> AuthResult<Vec<Certificate>> {
    let mut cursor = std::io::Cursor::new(pem);
    rustls_pemfile::certs(&mut cursor)
        .map(|certs| certs.into_iter().map(Certificate).collect())
        .map_err(|e| AuthError::CryptoError(format!("gagal parse certificate PEM: {e}")))
}

fn load_key(pem: &[u8]) -> AuthResult<PrivateKey> {
    let mut cursor = std::io::Cursor::new(pem);
    let keys = rustls_pemfile::pkcs8_private_keys(&mut cursor)
        .map_err(|e| AuthError::CryptoError(format!("gagal parse private key PEM: {e}")))?;

    keys.into_iter()
        .next()
        .map(PrivateKey)
        .ok_or_else(|| AuthError::CryptoError("tidak ada private key ditemukan di PEM".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_source_id() {
        assert_eq!(
            extract_source_id_from_cert("ats-client-web-app-01"),
            Some("web-app-01".to_string())
        );
        assert_eq!(
            extract_source_id_from_cert("web-app-01"), // tanpa prefix
            None
        );
        assert_eq!(
            extract_source_id_from_cert("ats-client-backend-svc"),
            Some("backend-svc".to_string())
        );
    }
}