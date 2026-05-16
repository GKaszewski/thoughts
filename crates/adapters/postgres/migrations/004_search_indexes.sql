CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE INDEX IF NOT EXISTS idx_thoughts_content_trgm
    ON thoughts USING GIN(content gin_trgm_ops);

CREATE INDEX IF NOT EXISTS idx_users_username_trgm
    ON users USING GIN(username gin_trgm_ops);

CREATE INDEX IF NOT EXISTS idx_users_display_name_trgm
    ON users USING GIN(display_name gin_trgm_ops)
    WHERE display_name IS NOT NULL;
