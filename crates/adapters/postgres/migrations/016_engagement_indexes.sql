-- Indexes for feed engagement counts and sorting.
-- likes and boosts are joined/counted per thought on every feed query.
-- thoughts(in_reply_to_id) is scanned for reply_count.
CREATE INDEX IF NOT EXISTS idx_likes_thought_id         ON likes(thought_id);
CREATE INDEX IF NOT EXISTS idx_boosts_thought_id        ON boosts(thought_id);
CREATE INDEX IF NOT EXISTS idx_thoughts_in_reply_to_id  ON thoughts(in_reply_to_id) WHERE in_reply_to_id IS NOT NULL;

-- Viewer-context lookups: "did I like/boost this?"
CREATE INDEX IF NOT EXISTS idx_likes_user_thought        ON likes(user_id, thought_id);
CREATE INDEX IF NOT EXISTS idx_boosts_user_thought       ON boosts(user_id, thought_id);
