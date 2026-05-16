CREATE TABLE outbox_events (
    seq          BIGSERIAL   PRIMARY KEY,
    aggregate_id UUID        NOT NULL,
    event_type   TEXT        NOT NULL,
    payload      JSONB       NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    delivered    BOOLEAN     NOT NULL DEFAULT false,
    delivered_at TIMESTAMPTZ
);
CREATE INDEX outbox_events_pending_idx ON outbox_events (seq) WHERE delivered = false;
