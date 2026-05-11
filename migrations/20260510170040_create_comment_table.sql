-- Add migration script here
CREATE TABLE IF NOT EXISTS comment (
    id BIGSERIAL PRIMARY KEY,
    paste_id BIGINT NOT NULL REFERENCES paste(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    author TEXT,
    created_at BIGINT NOT NULL,
    from_row BIGINT NOT NULL,
    from_column BIGINT NOT NULL,
    to_row BIGINT NOT NULL,
    to_column BIGINT NOT NULL,
    CONSTRAINT check_positive_positions CHECK (
        from_row >= 0 AND from_column >= 0 AND to_row >= 0 AND to_column >= 0
    )
);

CREATE INDEX IF NOT EXISTS idx_comment_paste_id ON comment(paste_id);
CREATE INDEX IF NOT EXISTS idx_comment_created_at ON comment(created_at);
CREATE INDEX IF NOT EXISTS idx_comment_paste_created ON comment(paste_id, created_at);
