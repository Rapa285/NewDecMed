# client-auditor (Tauri 2 + SvelteKit)

Desktop client for auditors: browse on-chain `LogRecord` metadata from the
`audit-trail` service, then inspect the underlying rotated log file
(newline-delimited `AuditRecord`s) fetched straight from IPFS.

This rebuilds `client-auditor` as a **SvelteKit** app (Svelte 5 runes,
`adapter-static`, Tailwind v4) so it matches the pattern used by
`client-ministry` and `client-hospital-tauri`, instead of the vanilla
HTML/JS approach used in `audit-trail-client`.

## Structure

```
client-auditor/
├── src/                        # SvelteKit frontend
│   ├── app.html / app.css
│   ├── lib/
│   │   ├── types.ts             # mirrors of the Rust types below
│   │   └── utils.ts             # cn, tryCatchAsVal, formatting helpers
│   └── routes/
│       ├── +layout.ts           # ssr=false, prerender=true (Tauri SPA)
│       ├── +layout.svelte       # app shell + toaster
│       ├── state.svelte.ts      # page state: log listing, pagination,
│       │                          settings, entry viewer
│       └── +page.svelte         # log table, settings modal, entry inspector
└── src-tauri/                   # Rust backend
    ├── src/
    │   ├── main.rs
    │   ├── lib.rs                # setup, plugin-store, command registration
    │   ├── commands.rs           # fetch_logs, fetch_log_entries,
    │   │                            get_settings, save_settings
    │   ├── api.rs                 # reqwest client: audit-trail + IPFS gateway
    │   ├── types.rs                # LogMetadata / LogRecord / AuditLogEntry / AppSettings
    │   ├── state.rs                 # AppState (http client + settings)
    │   └── error.rs                 # ClientError
    ├── capabilities/default.json   # core + opener + store permissions
    └── tauri.conf.json
```

## What it calls

- `GET {audit_trail_base_url}/api/logs?cursor=&limit=` — paginated
  `LogRecord` metadata (same contract `audit-trail-client` assumes; see
  `audit-trail/src/handlers.rs::get_logs`).
- `GET {ipfs_gateway_base_url}/ipfs/{cid}` — the rotated log file for a
  given `LogRecord`, parsed client-side as newline-delimited JSON
  `AuditRecord`s (see `audit-trail/src/types.rs::AuditRecord`).

Both URLs are configurable from the in-app Settings modal and persisted
via `tauri-plugin-store` (`settings.json` in the app data dir), same
approach as `audit-trail-client`.

## Known gap

Fetching individual `LogRecord`s currently depends on the same
`get_owned_objects` limitation tracked for the backend (it doesn't
surface `Immutable`/`Shared` objects — see project notes on Option
A/B/C for indexing `LogRecord`). Once the backend's `/api/logs` returns
live data under whichever fix is chosen, this client needs no changes.

## Running

```bash
npm install
npm run tauri dev
```
