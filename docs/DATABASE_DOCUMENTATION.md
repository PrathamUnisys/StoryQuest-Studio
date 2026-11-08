# StoryQuest Studio — SQLite Database Schema & Persistence Guide

## Overview
StoryQuest Studio uses a local-first **SQLite 3** database (`storyquest.db`) managed by the Rust `rusqlite` crate (with the `bundled` and `backup` features enabled).

### Crash & Power-Loss Safety
- Foreign key integrity is enforced on every connection via `PRAGMA foreign_keys = ON;`.
- Synchronous durability is set via `PRAGMA synchronous = FULL;`, ensuring task progress, focus session timers, and XP ledger events are flushed to disk immediately and never lost if Windows shuts down unexpectedly.
- Online hot backups are supported at any time via SQLite's `VACUUM INTO 'Backups/storyquest_backup_<timestamp>.db'` command (`cmd_backup_database`).

## Complete 21-Table Reference

| # | Table Name | Primary Key | Purpose & Key Columns |
| :- | :--- | :--- | :--- |
| 1 | `users` | `id` | Solo creator profile, `current_level`, `total_xp`, `active_title`, `active_theme`, and `daily_xp_custom_tasks` anti-exploit counter. |
| 2 | `settings` | `key` | Application settings (`start_with_windows`, `start_minimized`, `minimize_to_tray`, `quiet_hours_start`, `quiet_hours_end`, `projects_root_dir`). |
| 3 | `campaigns` | `id` | Animated series containers (`title`, `synopsis`, `genre`, `visual_style`, `world_info`, `status`, `project_folder_path`, `progress_percent`). |
| 4 | `episodes` | `id` | Individual episodes (`campaign_id`, `episode_number`, `title`, `premise`, `narrative_structure`, `current_stage`, `roadmap_approved`, `youtube_url`). |
| 5 | `tasks` | `id` | Production quests across Stages 1–15 (`stage_number`, `title`, `objective`, `required_input`, `ai_instructions`, `human_responsibilities`, `required_software`, `expected_output`, `estimated_effort_mins`, `actual_effort_mins`, `definition_of_done`, `completion_requirement_type`, `xp_reward`, `status`). |
| 6 | `task_dependencies` | `(task_id, depends_on_task_id)` | Directed Acyclic Graph (DAG) edges controlling when `locked` quests transition to `available`. |
| 7 | `milestones` | `id` | Stage & episode review gates (`stage_number`, `xp_reward`, `status`, `due_date`). |
| 8 | `xp_events` | `id` | **Idempotent XP Ledger** (`idempotency_key UNIQUE`, `source_type`, `source_id`, `xp_amount`, `skill_category`). Prevents duplicate XP awards. |
| 9 | `levels` | `level` | Progressive 10-level curve (`Level 1: Apprentice` at 0 XP through `Level 10: Master Storyteller` at 4,600 XP). |
| 10 | `achievements` | `id` | Unlockable badges (`code UNIQUE`, `title`, `xp_bonus`, `badge_svg_key`, `cosmetic_reward`, `unlocked`, `progress_current`, `progress_target`). |
| 11 | `streak_records` | `id` | Daily habit state (`current_streak`, `best_streak`, `streak_freezes_available`, `weekly_rest_day`, `vacation_mode`, `last_productive_date`). |
| 12 | `focus_sessions` | `id` | Timed work sessions (`task_id`, `planned_duration_mins`, `elapsed_seconds`, `status`, `quick_notes`, `outcome_summary`). |
| 13 | `reminders` | `id` | Scheduled Windows reminders (`reminder_type`, `schedule_time`, `days_of_week_json`, `enabled`, `snoozed_until`, `windows_task_name UNIQUE`). |
| 14 | `notification_history` | `id` | Log of toast notifications (`delivery_status`, `user_action`, `triggered_at`). |
| 15 | `assets` | `id` | Workshop assets across all 11 sections (`workshop_section`, `name`, `file_path`, `version`, `status`). |
| 16 | `characters` | `id` | Series character profiles (`name`, `role`, `voice_profile`, `visual_prompt`, `rig_status`, `rig_file_path`). |
| 17 | `environments` | `id` | Scene backgrounds & lighting moods (`name`, `lighting_mood`, `asset_folder_path`, `status`). |
| 18 | `animation_libraries` | `id` | Reusable animation clips (`clip_name`, `action_type`, `frame_count`, `fps`, `software_used`, `file_path`). |
| 19 | `ai_provider_configs` | `id` | Provider endpoints (`provider_name`, `base_url`, `default_model`, `credential_key_name` pointing to Windows Credential Manager). |
| 20 | `agent_connections` | `id` | External DCC & MCP integrations (`name`, `connection_type`, `executable_path`, `mcp_endpoint`, `requires_approval_for_writes`, `status`). |
| 21 | `agent_execution_logs` | `id` | Immutable audit log of external tool invocations (`action_type`, `command_payload_json`, `human_approved`, `execution_status`, `output_log`). |
