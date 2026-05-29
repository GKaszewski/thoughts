ALTER TABLE federation_following
  ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'accepted';
