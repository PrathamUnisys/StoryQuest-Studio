-- 005_gamification_notifications
-- M3: durable achievement notifications + streak controls.
-- Fix CHARACTER_CREATOR seed inconsistency: manual achievement must not show a progress bar.
UPDATE achievements
   SET progress_available = 0
 WHERE code = 'CHARACTER_CREATOR';

-- Durable notification store so unlocks survive restarts, display exactly once,
-- and support acknowledgement. Backfill produces one summary notification.
CREATE TABLE IF NOT EXISTS achievement_notifications (
    id                 TEXT PRIMARY KEY,
    user_id            TEXT NOT NULL REFERENCES users(id),
    achievement_id     TEXT NULL REFERENCES achievements(id),
    kind               TEXT NOT NULL CHECK (kind IN ('unlock', 'backfill_summary')),
    title              TEXT NOT NULL,
    body               TEXT NOT NULL,
    items_json         TEXT NOT NULL DEFAULT '[]',
    created_at         TEXT NOT NULL DEFAULT (datetime('now')),
    presented_at       TEXT NULL,
    acknowledged_at    TEXT NULL,
    dedupe_key         TEXT NOT NULL UNIQUE
);

CREATE INDEX IF NOT EXISTS idx_achievement_notifications_user_ack
    ON achievement_notifications(user_id, acknowledged_at, created_at);
