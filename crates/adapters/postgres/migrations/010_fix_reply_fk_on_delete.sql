-- Change in_reply_to_id FK from RESTRICT (default) to SET NULL.
-- Previously, deleting a thought that had replies raised a FK violation.
-- With SET NULL, deleting a thought orphans its replies (they survive but
-- lose their parent reference), which is the correct semantic for a
-- threaded social app.
ALTER TABLE thoughts
    DROP CONSTRAINT IF EXISTS thoughts_in_reply_to_id_fkey;

ALTER TABLE thoughts
    ADD CONSTRAINT thoughts_in_reply_to_id_fkey
    FOREIGN KEY (in_reply_to_id) REFERENCES thoughts(id) ON DELETE SET NULL;
