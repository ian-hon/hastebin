-- Add migration script here
-- Only alter the column if it's not already BYTEA
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'paste'
        AND column_name = 'content'
        AND data_type = 'text'
    ) THEN
        ALTER TABLE paste ALTER COLUMN content TYPE BYTEA USING content::bytea;
    END IF;
END $$;