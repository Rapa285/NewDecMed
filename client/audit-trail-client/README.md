# Audit Trail Log Viewer (Tauri)

Desktop client untuk melihat log metadata yang sudah tercatat di IOTA oleh
service `audit-trail`. Backend Rust (via Tauri) melakukan HTTP GET ke
endpoint audit-trail; frontend hanya menampilkan hasilnya.

## Struktur

```
audit-trail-client/
├── index.html          # shell UI
├── src/                # frontend (vanilla JS, tanpa framework)
│   ├── main.js
│   └── style.css
└── src-tauri/           # backend Rust
    ├── src/
    │   ├── main.rs      # entry point binary
    │   ├── lib.rs        # setup Tauri, plugin store, register commands
    │   ├── commands.rs   # tauri::command yang dipanggil dari frontend
    │   ├── api.rs         # HTTP client ke audit-trail (reqwest)
    │   ├── models.rs      # struct request/response
    │   ├── state.rs       # AppState (http client + settings)
    │   └── error.rs       # error type
    ├── capabilities/default.json  # permission plugin store (Tauri v2)
    └── tauri.conf.json
```

## Kontrak API yang diasumsikan

Karena endpoint di `audit-trail` belum diimplementasikan, client ini
mengasumsikan kontrak berikut. **Sesuaikan `src-tauri/src/api.rs` dan
`src-tauri/src/models.rs`** begitu endpoint aslinya sudah jadi kalau
bentuknya berbeda.

```
GET {base_url}/api/logs?cursor=<opaque>&limit=<n>

200 OK
{
  "data": [
    {
      "object_id": "0x...",
      "metadata": {
        "version": "1.0",
        "log_sequence_number": 0,
        "rotation_timestamp": "2025-01-01T00:00:00Z",
        "ipfs_cid": "Qm...",
        "file_hash": "...",
        "first_record_hash": "...",
        "final_record_hash": "...",
        "record_count": 42,
        "prev_tx_digest": "..." // atau null
      },
      "tx_digest": "..." // opsional
    }
  ],
  "next_cursor": "...",     // null jika tidak ada halaman berikutnya
  "has_next_page": true
}
```

Field `metadata` dibuat sepadan dengan struct `IotaLogMetadata` di
`audit-trail/src/iota_client.rs`, dan `object_id` sepadan dengan
`LogRecordOnChain.object_id`. Kalau nanti endpoint backend memakai
`get_log_records()` yang sudah ada di `IotaLogClient`, tinggal
bungkus hasilnya jadi bentuk `LogsResponse` di atas (atau ubah
`LogsResponse`/parsing di `api.rs` sesuai bentuk asli).

Jika endpoint backend tidak mendukung pagination cursor-based, cukup
selalu kembalikan `has_next_page: false` — tombol "Muat lebih banyak"
otomatis tidak akan muncul.

## Konfigurasi base URL

Base URL audit-trail disimpan lewat `tauri-plugin-store`
(`settings.json` di app data dir), diatur dari UI lewat tombol ⚙︎.
Default: `http://localhost:3000` (sesuai `audit-trail/.env` → `PORT=3000`).

## Menjalankan

Prasyarat: Node.js, Rust toolchain, dan dependency sistem Tauri v2
(lihat https://v2.tauri.app/start/prerequisites/).

```bash
npm install
npm run tauri dev
```

Build production:

```bash
npm run tauri build
```

## Catatan

- Ikon aplikasi (`src-tauri/icons/`) belum disertakan — generate dengan
  `npm run tauri icon path/to/logo.png` sebelum build production, atau
  hapus field `bundle.icon` di `tauri.conf.json` untuk dev build.
- CORS: pastikan endpoint `audit-trail` mengizinkan origin `tauri://localhost`
  (desktop) / `http://localhost:1420` (dev), atau tambahkan header
  `Access-Control-Allow-Origin` yang sesuai di server Axum-nya.
