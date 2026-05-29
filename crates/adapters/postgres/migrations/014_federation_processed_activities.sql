CREATE TABLE IF NOT EXISTS federation_processed_activities (
    activity_id  TEXT PRIMARY KEY,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_fed_processed_activities_at
    ON federation_processed_activities(processed_at);
