ALTER TABLE remote_actors
    ADD COLUMN IF NOT EXISTS bio           TEXT,
    ADD COLUMN IF NOT EXISTS banner_url    TEXT,
    ADD COLUMN IF NOT EXISTS followers_url TEXT,
    ADD COLUMN IF NOT EXISTS following_url TEXT,
    ADD COLUMN IF NOT EXISTS also_known_as TEXT[];
