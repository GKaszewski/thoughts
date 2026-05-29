ALTER TABLE remote_actors ADD COLUMN IF NOT EXISTS attachment JSONB DEFAULT '[]'::jsonb;
