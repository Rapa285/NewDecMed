// audit-trail/src/bin/generate_als_key.rs

use p256::{
    ecdsa::SigningKey,
    pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding},
};
// Gunakan OsRng dari rand_core 0.6 langsung,
// bukan dari rand (yang sudah versi 0.9)
use rand_core::OsRng;

fn main() {
    // OsRng dari rand_core 0.6 kompatibel dengan p256
    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = signing_key.verifying_key();

    let private_pem = signing_key
        .to_pkcs8_pem(LineEnding::LF)
        .unwrap();

    let public_pem = verifying_key
        .to_public_key_pem(LineEnding::LF)
        .unwrap();

    // Zeroizing<String> perlu di-deref ke &str untuk println
    println!("=== PRIVATE KEY (simpan di ALS_PRIVATE_KEY_PEM di .env) ===");
    println!("{}", private_pem.as_str());

    println!("\n=== PUBLIC KEY (hardcode di ALS_SERVER_PUBLIC_KEY_PEM di constants.rs) ===");
    println!("{public_pem}");
}