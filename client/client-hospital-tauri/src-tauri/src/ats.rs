use ed25519_dalek::{Signature, Signer, Verifier, SigningKey, VerifyingKey};
use rand::rngs::OsRng;

// --- Modul ATS ---
pub mod ats_module {

    /// Menghasilkan pasangan kunci (Private/Signing Key dan Public/Verifying Key)
    pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        
        (signing_key, verifying_key)
    }

    /// Menandatangani pesan menggunakan Private Key (SigningKey)
    pub fn sign_message(signing_key: &SigningKey, message: &[u8]) -> Signature {
        signing_key.sign(message)
    }

    /// Memverifikasi keaslian pesan menggunakan Public Key (VerifyingKey)
    pub fn verify_signature(
        verifying_key: &VerifyingKey, 
        message: &[u8], 
        signature: &Signature
    ) -> bool {
        // is_ok() mengembalikan true jika verifikasi berhasil tanpa error
        verifying_key.verify(message, signature).is_ok()
    }

    
}