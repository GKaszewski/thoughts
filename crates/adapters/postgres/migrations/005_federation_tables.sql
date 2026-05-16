-- Add avatar_url and outbox_url to remote_actors (FederationRepository::RemoteActor needs them)
ALTER TABLE remote_actors
    ADD COLUMN IF NOT EXISTS avatar_url  TEXT,
    ADD COLUMN IF NOT EXISTS outbox_url  TEXT;

-- Federation followers: remote actors following local users
CREATE TABLE IF NOT EXISTS federation_followers (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    local_user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    remote_actor_url TEXT NOT NULL,
    status           TEXT NOT NULL DEFAULT 'pending',
    follow_activity_id TEXT NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (local_user_id, remote_actor_url)
);

-- Federation following: local users following remote actors
CREATE TABLE IF NOT EXISTS federation_following (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    local_user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    remote_actor_url TEXT NOT NULL,
    follow_activity_id TEXT NOT NULL,
    outbox_url       TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (local_user_id, remote_actor_url)
);

-- Announces (boosts of remote objects via AP)
CREATE TABLE IF NOT EXISTS federation_announces (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    activity_id  TEXT NOT NULL UNIQUE,
    object_url   TEXT NOT NULL,
    actor_url    TEXT NOT NULL,
    announced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Blocked domains (instance-level)
CREATE TABLE IF NOT EXISTS federation_blocked_domains (
    domain     TEXT PRIMARY KEY,
    reason     TEXT,
    blocked_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Blocked actors (per local user)
CREATE TABLE IF NOT EXISTS federation_blocked_actors (
    local_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    actor_url     TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (local_user_id, actor_url)
);

CREATE INDEX IF NOT EXISTS idx_fed_followers_user   ON federation_followers(local_user_id);
CREATE INDEX IF NOT EXISTS idx_fed_following_user   ON federation_following(local_user_id);
CREATE INDEX IF NOT EXISTS idx_fed_announces_object ON federation_announces(object_url);
