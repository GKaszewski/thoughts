CREATE TABLE remote_actor_connections (
    id                     UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_url              TEXT        NOT NULL,
    connection_type        TEXT        NOT NULL,
    page                   INT         NOT NULL,
    connected_actor_url    TEXT        NOT NULL,
    connected_handle       TEXT        NOT NULL,
    connected_display_name TEXT,
    connected_avatar_url   TEXT,
    fetched_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(actor_url, connection_type, page, connected_actor_url)
);
CREATE INDEX ON remote_actor_connections(actor_url, connection_type, page, fetched_at);
