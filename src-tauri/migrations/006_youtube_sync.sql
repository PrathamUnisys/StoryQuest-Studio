-- 006_youtube_sync
-- M4: YouTube Data/Analytics sync snapshots. Tokens are never stored here;
-- OAuth tokens + client credentials live in Windows Credential Manager.

CREATE TABLE IF NOT EXISTS youtube_channel_snapshots (
    id                      TEXT PRIMARY KEY,
    user_id                 TEXT NOT NULL REFERENCES users(id),
    channel_id              TEXT NOT NULL,
    channel_title           TEXT NOT NULL DEFAULT '',
    subscriber_count        INTEGER NULL,
    hidden_subscriber_count INTEGER NOT NULL DEFAULT 0,
    view_count              INTEGER NOT NULL DEFAULT 0,
    video_count             INTEGER NOT NULL DEFAULT 0,
    synced_at               TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_youtube_channel_snapshots_user_time
    ON youtube_channel_snapshots(user_id, synced_at);

CREATE TABLE IF NOT EXISTS youtube_analytics_snapshots (
    id                          TEXT PRIMARY KEY,
    user_id                     TEXT NOT NULL REFERENCES users(id),
    period_start                TEXT NOT NULL,
    period_end                  TEXT NOT NULL,
    views                       INTEGER NOT NULL DEFAULT 0,
    estimated_minutes_watched   INTEGER NOT NULL DEFAULT 0,
    subscribers_gained          INTEGER NOT NULL DEFAULT 0,
    subscribers_lost            INTEGER NOT NULL DEFAULT 0,
    synced_at                   TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_youtube_analytics_snapshots_user_time
    ON youtube_analytics_snapshots(user_id, synced_at);

INSERT OR IGNORE INTO settings (key, value, category) VALUES
    ('youtube_channel_id', '', 'youtube'),
    ('youtube_sync_enabled', 'false', 'youtube'),
    ('youtube_sync_interval_hours', '24', 'youtube'),
    ('youtube_last_sync_at', '', 'youtube'),
    ('youtube_connection_status', 'not_connected', 'youtube'),
    ('youtube_channel_title', '', 'youtube');
