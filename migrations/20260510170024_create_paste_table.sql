-- Add migration script here
CREATE TABLE IF NOT EXISTS paste (
    id BIGSERIAL PRIMARY KEY,
    content TEXT NOT NULL,
    title TEXT,
    author TEXT,
    checksum_passphrase TEXT,
    views BIGINT NOT NULL DEFAULT 0,
    comments_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at BIGINT NOT NULL,
    expires_at BIGINT,
    forked_from BIGINT REFERENCES paste(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_paste_created_at ON paste(created_at);
CREATE INDEX IF NOT EXISTS idx_paste_expires_at ON paste(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_paste_forked_from ON paste(forked_from) WHERE forked_from IS NOT NULL;
