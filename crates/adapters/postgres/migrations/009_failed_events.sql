CREATE TABLE failed_events (
    id           UUID        NOT NULL DEFAULT gen_random_uuid(),
    event_type   TEXT        NOT NULL,
    payload      JSONB       NOT NULL,
    failed_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    retry_at     TIMESTAMPTZ NOT NULL,
    retry_count  INT         NOT NULL DEFAULT 0,
    last_error   TEXT        NOT NULL,

    CONSTRAINT failed_events_pkey PRIMARY KEY (id)
);

CREATE INDEX failed_events_due_idx
    ON failed_events (retry_at)
    WHERE retry_count < 3;
