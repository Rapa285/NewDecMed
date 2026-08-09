CREATE TABLE audit_trail (
    id            BIGSERIAL PRIMARY KEY,
    event_time    TIMESTAMPTZ NOT NULL DEFAULT now(),
    actor_id      TEXT NOT NULL,        -- siapa/perangkat mana yang melakukan aksi
    action        TEXT NOT NULL,        -- CREATE, READ, UPDATE, DELETE, ACCESS, dll
    resource_type TEXT,                 -- jenis objek/data yang diakses
    resource_id   TEXT,                 -- id objek/data spesifik
    outcome       TEXT NOT NULL,        -- SUCCESS / FAILURE
    ipfs_cid      TEXT NOT NULL,        -- pointer ke konten detail di IPFS
    iota_block_id TEXT NOT NULL,        -- referensi bukti integritas di IOTA
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index buat query yang paling sering dipakai
CREATE INDEX idx_audit_event_time ON audit_trail (event_time);
CREATE INDEX idx_audit_actor      ON audit_trail (actor_id);
CREATE INDEX idx_audit_resource   ON audit_trail (resource_id);
CREATE INDEX idx_audit_action     ON audit_trail (action);