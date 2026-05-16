-- Remote ActivityPub posts can exceed 128 characters.
-- The 128-char limit is enforced at the application layer for local posts only.
ALTER TABLE thoughts ALTER COLUMN content TYPE TEXT;
