"""StoryQuest Studio — Milestone 1 verification suite.

This suite verifies the SQLite schema and the foundation migration only.
It never overwrites the live developer database.
"""

from __future__ import annotations

import pathlib
import sqlite3
import tempfile
import unittest

PROJECT_ROOT = pathlib.Path(__file__).resolve().parent.parent
MIGRATIONS_DIR = PROJECT_ROOT / "src-tauri" / "migrations"

MIGRATION_FILES = [
    MIGRATIONS_DIR / "001_initial_schema.sql",
    MIGRATIONS_DIR / "002_seed_data.sql",
    MIGRATIONS_DIR / "003_foundation.sql",
    MIGRATIONS_DIR / "004_repair_achievement_seed.sql",
    MIGRATIONS_DIR / "005_achievement_notifications.sql",
]

REQUIRED_TABLES = {
    "users",
    "levels",
    "skill_categories",
    "xp_events",
    "campaigns",
    "episodes",
    "tasks",
    "task_dependencies",
    "milestones",
    "achievements",
    "streak_records",
    "reminders",
    "settings",
    "assets",
    "characters",
    "environments",
    "animation_libraries",
    "focus_sessions",
    "notification_history",
    "ai_provider_configs",
    "agent_connections",
    "agent_execution_logs",
}

RANK_MILESTONES = [0, 1, 3, 5, 10, 15, 25, 40, 60, 100, 150, 250, 500]


def init_db(db_path: pathlib.Path) -> sqlite3.Connection:
    conn = sqlite3.connect(str(db_path))
    conn.execute("PRAGMA foreign_keys = ON;")
    for migration in MIGRATION_FILES:
        conn.executescript(migration.read_text(encoding="utf-8"))
    conn.commit()
    return conn


class Milestone1FoundationTests(unittest.TestCase):
    def setUp(self):
        self.tmp_dir = tempfile.TemporaryDirectory()
        self.db_path = pathlib.Path(self.tmp_dir.name) / "storyquest_test.db"
        self.conn = init_db(self.db_path)
        self.conn.row_factory = sqlite3.Row

    def tearDown(self):
        self.conn.close()
        self.tmp_dir.cleanup()

    def test_required_tables_exist(self):
        rows = self.conn.execute(
            "SELECT name FROM sqlite_master WHERE type = 'table'"
        ).fetchall()
        table_names = {row["name"] for row in rows}
        missing = REQUIRED_TABLES - table_names
        self.assertFalse(missing, f"Missing tables: {sorted(missing)}")
        self.assertIn("creator_ranks", table_names)
        self.assertIn("creator_progress", table_names)

    def test_fabricated_profile_progress_is_reset(self):
        profile = self.conn.execute(
            "SELECT display_name, current_level, total_xp, active_title "
            "FROM users WHERE id = 'user_solo_01'"
        ).fetchone()
        self.assertEqual(profile["display_name"], "Solo Creator")
        self.assertEqual(profile["current_level"], 1)
        self.assertEqual(profile["total_xp"], 0)
        self.assertEqual(profile["active_title"], "Aspiring Creator")

        streak = self.conn.execute(
            "SELECT current_streak, best_streak FROM streak_records "
            "WHERE user_id = 'user_solo_01'"
        ).fetchone()
        self.assertEqual(streak["current_streak"], 0)
        self.assertEqual(streak["best_streak"], 0)

        self.assertNotEqual(profile["display_name"], "Aria Vance")
        self.assertNotEqual(profile["total_xp"], 285)

    def test_creator_ranks_are_published_video_based(self):
        ranks = self.conn.execute(
            "SELECT level, rank_name, min_published_videos FROM creator_ranks "
            "ORDER BY level"
        ).fetchall()
        self.assertEqual(len(ranks), 13)
        self.assertEqual([row["min_published_videos"] for row in ranks], RANK_MILESTONES)
        self.assertEqual(ranks[0]["rank_name"], "Aspiring Creator")
        self.assertEqual(ranks[-1]["rank_name"], "Legendary Creator")

    def test_creator_progress_starts_at_rank_one(self):
        progress = self.conn.execute(
            "SELECT rank_level, published_videos_total, subscriber_count "
            "FROM creator_progress WHERE user_id = 'user_solo_01'"
        ).fetchone()
        self.assertEqual(progress["rank_level"], 1)
        self.assertEqual(progress["published_videos_total"], 0)
        self.assertIsNone(progress["subscriber_count"])

    def test_starter_campaign_is_sample_and_not_started(self):
        campaign = self.conn.execute(
            "SELECT title, status, progress_percent, is_sample FROM campaigns "
            "WHERE id = 'camp_01'"
        ).fetchone()
        self.assertEqual(campaign["title"], "Chronicles of Aethelgard")
        self.assertEqual(campaign["status"], "planning")
        self.assertEqual(campaign["progress_percent"], 0.0)
        self.assertEqual(campaign["is_sample"], 1)

        episode = self.conn.execute(
            "SELECT current_stage, roadmap_approved, status FROM episodes "
            "WHERE id = 'ep_01'"
        ).fetchone()
        self.assertEqual(episode["current_stage"], 1)
        self.assertEqual(episode["roadmap_approved"], 0)
        self.assertEqual(episode["status"], "draft")

    def test_fifteen_stage_roadmap_and_dependencies(self):
        tasks = self.conn.execute(
            "SELECT id, stage_number, status FROM tasks WHERE episode_id = 'ep_01' "
            "ORDER BY stage_number"
        ).fetchall()
        self.assertEqual(len(tasks), 15)
        self.assertEqual(tasks[0]["status"], "available")
        self.assertTrue(all(row["status"] == "locked" for row in tasks[1:]))

        dependencies = self.conn.execute(
            "SELECT COUNT(*) AS count FROM task_dependencies"
        ).fetchone()["count"]
        self.assertEqual(dependencies, 14)

    def test_achievements_are_seeded_locked_with_conditions(self):
        achievements = self.conn.execute(
            "SELECT code, unlocked, condition_type, progress_target FROM achievements"
        ).fetchall()
        self.assertGreaterEqual(len(achievements), 20)
        self.assertTrue(all(row["unlocked"] == 0 for row in achievements))
        self.assertTrue(
            any(row["condition_type"] == "published_count" for row in achievements)
        )
        self.assertTrue(any(row["condition_type"] == "subscribers" for row in achievements))

        first_story = self.conn.execute(
            "SELECT unlocked, unlocked_at FROM achievements WHERE code = 'FIRST_STORY'"
        ).fetchone()
        self.assertEqual(first_story["unlocked"], 0)
        self.assertIsNone(first_story["unlocked_at"])

    def test_settings_are_real_defaults(self):
        settings = {
            row["key"]: row["value"]
            for row in self.conn.execute("SELECT key, value FROM settings").fetchall()
        }
        self.assertEqual(settings["start_with_windows"], "false")
        self.assertEqual(settings["minimize_to_tray"], "true")
        self.assertEqual(settings["onboarding_completed"], "false")
        self.assertEqual(settings["youtube_sync_enabled"], "false")
        self.assertEqual(settings["youtube_connection_status"], "not_connected")
        self.assertEqual(settings["daily_quest_min"], "3")
        self.assertEqual(settings["daily_quest_max"], "7")

    def test_no_hardcoded_api_keys(self):
        providers = self.conn.execute(
            "SELECT provider_name, credential_key_name FROM ai_provider_configs"
        ).fetchall()
        self.assertEqual(len(providers), 4)
        for provider in providers:
            self.assertFalse(provider["credential_key_name"].startswith("sk-"))

    def test_online_backup_contains_the_seeded_database(self):
        backup_path = pathlib.Path(self.tmp_dir.name) / "backup_storyquest.db"
        self.conn.execute(f"VACUUM INTO '{backup_path.as_posix()}'")
        self.assertTrue(backup_path.exists())

        backup = sqlite3.connect(str(backup_path))
        try:
            count = backup.execute("SELECT COUNT(*) FROM tasks").fetchone()[0]
            ranks = backup.execute("SELECT COUNT(*) FROM creator_ranks").fetchone()[0]
        finally:
            backup.close()
        self.assertEqual(count, 15)
        self.assertEqual(ranks, 13)

    def test_xp_ledger_unique_idempotency_key(self):
        self.conn.execute(
            "INSERT INTO xp_events "
            "(id, user_id, idempotency_key, source_type, source_id, xp_amount, description) "
            "VALUES ('xp_test_1', 'user_solo_01', 'task_complete::test', "
            "'task_completion', 'test', 10, 'test')"
        )
        with self.assertRaises(sqlite3.IntegrityError):
            self.conn.execute(
                "INSERT INTO xp_events "
                "(id, user_id, idempotency_key, source_type, source_id, xp_amount, description) "
                "VALUES ('xp_test_2', 'user_solo_01', 'task_complete::test', "
                "'task_completion', 'test', 10, 'test')"
            )

    def test_character_creator_is_hidden_manual(self):
        row = self.conn.execute(
            "SELECT condition_type, progress_available, is_hidden FROM achievements WHERE code = 'VOICE_OF_WORLD'"
        ).fetchone()
        self.assertEqual(row["condition_type"], "manual")
        self.assertEqual(row["progress_available"], 0)

    def test_notification_table_exists(self):
        tables = {r[0] for r in self.conn.execute(
            "SELECT name FROM sqlite_master WHERE type='table'"
        ).fetchall()}
        self.assertIn("achievement_notifications", tables)

    def test_notification_dedupe_key_unique(self):
        self.conn.execute(
            "INSERT INTO achievement_notifications (id, user_id, kind, title, body, items_json, dedupe_key) "
            "VALUES ('n1', 'user_solo_01', 'unlock', 'T', 'B', '[]', 'dedupe_1')"
        )
        with self.assertRaises(sqlite3.IntegrityError):
            self.conn.execute(
                "INSERT INTO achievement_notifications (id, user_id, kind, title, body, items_json, dedupe_key) "
                "VALUES ('n2', 'user_solo_01', 'unlock', 'T', 'B', '[]', 'dedupe_1')"
            )


if __name__ == "__main__":
    unittest.main(verbosity=2)
