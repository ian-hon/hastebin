-- Add migration script here
-- Add page_index column to comment table to support multi-page pastes
ALTER TABLE comment ADD COLUMN page_index BIGINT NOT NULL DEFAULT 0;

-- Add index for efficient querying by paste_id and page_index
CREATE INDEX IF NOT EXISTS idx_comment_paste_page ON comment(paste_id, page_index);

-- Add check constraint to ensure page_index is non-negative
ALTER TABLE comment ADD CONSTRAINT check_positive_page_index CHECK (page_index >= 0);
