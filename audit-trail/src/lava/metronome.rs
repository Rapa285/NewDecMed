// Metronome Timer
// Implementasi dari LAVA Section 3.4 (parameter d).
// Setiap d detik, jika tidak ada event masuk, metronome inject dummy entry
// ke dalam engine. Ini mencegah truncation attack karena verifier tahu
// bahwa dalam setiap interval d detik harus ada minimal 1 entry.

use std::sync::Arc;
use tokio::{
    sync::Mutex,
    time::{interval, Duration},
};

use crate::lava::{engine::LavaEngine, error::LavaResult};

/// Jalankan metronome sebagai background task.
/// Handle-nya bisa di-abort untuk stop gracefully.
pub fn spawn(engine: Arc<Mutex<LavaEngine>>, interval_secs: u64) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(interval_secs));
        // Tick pertama terjadi langsung — skip agar tidak inject di awal
        ticker.tick().await;

        loop {
            ticker.tick().await;

            let mut eng = engine.lock().await;
            if let Err(e) = eng.inject_metronome() {
                // Log error tapi jangan stop metronome — lanjutkan tick berikutnya
                eprintln!("[metronome] error injecting entry: {e}");
            }
        }
    })
}

/// Wrapper sinkron untuk testing — inject satu metronome entry secara langsung
pub async fn inject_once(engine: Arc<Mutex<LavaEngine>>) -> LavaResult<()> {
    let mut eng = engine.lock().await;
    eng.inject_metronome()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lava::{engine::LavaEngine, types::LavaParams};
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_metronome_injects_entry() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let params = LavaParams {
            d: 1, // 1 detik untuk test
            ..Default::default()
        };
        let engine = Arc::new(Mutex::new(
            LavaEngine::new(params, tx).expect("engine harus berhasil dibuat"),
        ));

        inject_once(Arc::clone(&engine)).await.unwrap();

        let item = rx.try_recv().expect("harus ada item setelah inject");
        assert!(
            matches!(item, crate::lava::types::LogItem::Metronome(_)),
            "item harus bertipe Metronome"
        );
    }
}