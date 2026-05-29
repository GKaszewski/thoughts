ALTER TABLE users ADD COLUMN IF NOT EXISTS profile_fields JSONB DEFAULT '[]'::jsonb;
