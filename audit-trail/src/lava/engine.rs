// LAVA Engine
// Orkestrator utama yang mengkoordinasikan semua komponen:
// hash chain + auth engine + credential rotator + output queue.
// Implementasi dari Algorithm 2 di paper Bajramovic et al. 2023.

use chrono::Utc;
use serde_json::Value;
use tokio::sync::mpsc;

use crate::lava::{
    credential::CredentialManager,
    error::LavaResult,
    hash_chain::HashChain,
    types::{
        Authenticator, CredentialUpdate, LavaParams, LogEntry, LogItem,
        MetronomeEntry, VerificationAnchor,
    },
};

pub struct LavaEngine {
    params: LavaParams,
    hash_chain: HashChain,
    credentials: CredentialManager,
    /// Counter global — di-increment tiap log entry biasa
    entry_counter: u64,
    /// Queue output — semua LogItem dikirim ke sini
    output_tx: mpsc::UnboundedSender<LogItem>,
}

impl LavaEngine {
    pub fn new(
        params: LavaParams,
        output_tx: mpsc::UnboundedSender<LogItem>,
    ) -> LavaResult<Self> {
        params.validate()?;
        Ok(Self {
            params,
            hash_chain: HashChain::new(),
            credentials: CredentialManager::new()?,
            entry_counter: 0,
            output_tx,
        })
    }

    /// Public key credential awal — harus disimpan ke IOTA sebagai jangkar kepercayaan
    pub fn initial_public_key(&self) -> &str {
        self.credentials.current_public_key()
    }

    /// Public key long-term credential E — untuk fast-forward verification
    pub fn long_term_public_key(&self) -> &str {
        self.credentials.long_term_public_key()
    }

    /// Proses satu event masuk dari event source.
    /// Ini adalah entry point utama — dipanggil oleh Event Receiver.
    /// Urutan sesuai paper Algorithm 2 lines 15-27.
    pub fn process_event(&mut self, data: Value) -> LavaResult<()> {
        let i = self.entry_counter;

        // 1. Serialize data untuk hash
        let data_str = serde_json::to_string(&data)?;

        // 2. Advance hash chain: h_i = H(h_{i-1} || i || data)
        let new_hash = self.hash_chain.advance(i, &data_str);

        // 3. Buat log entry dan kirim ke output queue
        let entry = LogEntry {
            index: i,
            hash: new_hash.clone(),
            timestamp: Utc::now(),
            data,
        };
        self.send(LogItem::Entry(entry))?;

        // 4. Increment credential counter
        self.credentials.increment();
        self.entry_counter += 1;

        // 5. Cek apakah perlu credential rotation (setiap c entries)
        //    Harus sebelum auth agar authenticator menggunakan key baru jika rotation terjadi
        if self.credentials.needs_rotation(self.params.c) {
            self.do_credential_rotation()?;
        }

        // 6. Cek apakah perlu authenticator (setiap a entries)
        if self.entry_counter % self.params.a == 0 {
            self.do_authenticate()?;
        }

        // 7. Cek apakah perlu verification anchor (setiap e entries)
        if self.entry_counter % self.params.e == 0 {
            self.do_verification_anchor()?;
        }

        Ok(())
    }

    /// Inject metronome entry — dipanggil oleh MetronomeTimer setiap d detik.
    /// Metronome menggunakan counter yang sama dengan entry biasa
    /// sehingga gap yang ditinggalkan truncation attack bisa terdeteksi.
    pub fn inject_metronome(&mut self) -> LavaResult<()> {
        let i = self.entry_counter;

        // Metronome juga masuk ke hash chain
        let metronome_data = format!("metronome:{}", Utc::now().timestamp_millis());
        let new_hash = self.hash_chain.advance(i, &metronome_data);

        let entry = MetronomeEntry {
            index: i,
            hash: new_hash,
            timestamp: Utc::now(),
        };
        self.send(LogItem::Metronome(entry))?;

        // Metronome juga increment counter
        self.credentials.increment();
        self.entry_counter += 1;

        // Cek rotation dan auth sama seperti entry biasa
        if self.credentials.needs_rotation(self.params.c) {
            self.do_credential_rotation()?;
        }
        if self.entry_counter % self.params.a == 0 {
            self.do_authenticate()?;
        }

        Ok(())
    }

    // ── Private helpers ────────────────────────────────────────────────────────

    fn do_authenticate(&mut self) -> LavaResult<()> {
        let current_hash = self.hash_chain.current().to_string();
        let signature = self.credentials.sign(&current_hash);

        let auth = Authenticator {
            entry_index: self.entry_counter,
            hash: current_hash,
            signature,
        };
        self.send(LogItem::Authenticator(auth))
    }

    fn do_credential_rotation(&mut self) -> LavaResult<()> {
        let (new_pk, signature) = self.credentials.rotate()?;

        let update = CredentialUpdate {
            entry_index: self.entry_counter,
            new_public_key: new_pk,
            signature,
        };
        self.send(LogItem::CredentialUpdate(update))
    }

    fn do_verification_anchor(&mut self) -> LavaResult<()> {
        let current_hash = self.hash_chain.current().to_string();
        let current_pk = self.credentials.current_public_key().to_string();

        // Anchor ditandatangani oleh long-term key E
        // Data yang ditandatangani: hash || current_pk || entry_index
        let anchor_data = format!(
            "{}:{}:{}",
            current_hash, current_pk, self.entry_counter
        );
        let signature = self.credentials.sign_long_term(&anchor_data);

        let anchor = VerificationAnchor {
            entry_index: self.entry_counter,
            hash: current_hash,
            current_public_key: current_pk,
            signature,
        };
        self.send(LogItem::VerificationAnchor(anchor))
    }

    fn send(&self, item: LogItem) -> LavaResult<()> {
        self.output_tx
            .send(item)
            .map_err(|_| crate::lava::error::LavaError::CryptoError(
                "output channel tertutup".into(),
            ))
    }
}