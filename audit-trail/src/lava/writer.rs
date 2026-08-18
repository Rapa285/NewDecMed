// Output Writer
// Implementasi dari LAVA Section 4.1 (parameter b — communication efficiency).
// Writer menerima LogItem dari channel, mengumpulkan b item,
// lalu menulis semuanya ke file.log sekaligus (satu flush per batch).
// Setiap item ditulis sebagai satu baris JSON (newline-delimited JSON / NDJSON).

use std::path::PathBuf;
use tokio::{
    fs::OpenOptions,
    io::AsyncWriteExt,
    sync::mpsc,
};

use crate::lava::{error::LavaResult, types::LogItem};

pub struct LogWriter {
    /// Path ke file.log
    path: PathBuf,
    /// Ukuran batch sebelum flush (parameter b)
    batch_size: u64,
    /// Buffer sementara sebelum ditulis ke disk
    buffer: Vec<LogItem>,
    /// Total item yang sudah ditulis ke disk
    total_written: u64,
}

impl LogWriter {
    pub fn new(path: PathBuf, batch_size: u64) -> Self {
        Self {
            path,
            batch_size,
            buffer: Vec::new(),
            total_written: 0,
        }
    }

    /// Terima satu item, tambahkan ke buffer.
    /// Jika buffer penuh (>= b), flush ke disk.
    pub async fn push(&mut self, item: LogItem) -> LavaResult<()> {
        self.buffer.push(item);
        if self.buffer.len() as u64 >= self.batch_size {
            self.flush().await?;
        }
        Ok(())
    }

    /// Paksa flush semua item di buffer ke disk (dipanggil saat shutdown / rotation)
    pub async fn flush(&mut self) -> LavaResult<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await?;

        for item in self.buffer.drain(..) {
            let mut line = serde_json::to_string(&item)?;
            line.push('\n');
            file.write_all(line.as_bytes()).await?;
            self.total_written += 1;
        }

        file.flush().await?;
        Ok(())
    }

    pub fn total_written(&self) -> u64 {
        self.total_written
    }
}

/// Jalankan writer sebagai background task yang membaca dari channel.
/// Berhenti secara graceful ketika channel ditutup (sender di-drop).
pub async fn run_writer(
    mut rx: mpsc::UnboundedReceiver<LogItem>,
    path: PathBuf,
    batch_size: u64,
) -> LavaResult<u64> {
    let mut writer = LogWriter::new(path, batch_size);

    while let Some(item) = rx.recv().await {
        writer.push(item).await?;
    }

    // Flush sisa buffer saat channel ditutup
    writer.flush().await?;
    Ok(writer.total_written())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lava::types::{LogEntry, LogItem};
    use chrono::Utc;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_writer_flushes_on_batch() {
        let tmpfile = NamedTempFile::new().unwrap();
        let path = tmpfile.path().to_path_buf();
        let mut writer = LogWriter::new(path.clone(), 3);

        // Push 3 item — harus auto-flush
        for i in 0u64..3 {
            let item = LogItem::Entry(LogEntry {
                index: i,
                hash: format!("hash{i}"),
                timestamp: Utc::now(),
                data: serde_json::json!({ "event": format!("e{i}") }),
            });
            writer.push(item).await.unwrap();
        }

        // Baca file dan cek ada 3 baris
        let content = tokio::fs::read_to_string(&path).await.unwrap();
        assert_eq!(content.lines().count(), 3);
    }

    #[tokio::test]
    async fn test_writer_flushes_remainder_on_close() {
        let tmpfile = NamedTempFile::new().unwrap();
        let path = tmpfile.path().to_path_buf();
        let mut writer = LogWriter::new(path.clone(), 10); // batch 10, tapi hanya push 2

        for i in 0u64..2 {
            let item = LogItem::Entry(LogEntry {
                index: i,
                hash: format!("hash{i}"),
                timestamp: Utc::now(),
                data: serde_json::json!({}),
            });
            writer.push(item).await.unwrap();
        }
        writer.flush().await.unwrap(); // manual flush

        let content = tokio::fs::read_to_string(&path).await.unwrap();
        assert_eq!(content.lines().count(), 2);
    }
}