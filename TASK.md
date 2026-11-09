# StoryQuest Studio — Task Tracker

Single source of truth for "where we are." Updated after every work session.

## Current state (M6 — AI provider + roadmap)
**Status:** M1–M6 code complete and verified. Awaiting live acceptance (M5 reminder, M6 provider) then M7 installer smoke test.

Live database `StoryQuestProjects/storyquest.db` is at migration `006_youtube_sync`.
- User: `Solo Creator`, Level 1, 0 XP, 0 streak (fabricated Aria Vance data reset).
- Creator ranks: 13, thresholds 0/1/3/5/10/15/25/40/60/100/150/250/500 published videos.
- Achievements: 24 seeded, all locked until conditions are met. Auto-unlock engine active.
- CHARACTER_CREATOR fixed: `progress_available = 0` (manual achievement).
- Tasks: 15 in sample campaign `camp_01`; 1 available, 14 locked.
- YouTube: `connected` (live, accepted 2026-11-09). OAuth PKCE+loopback, Data API + Analytics sync, migration 006 snapshots, `.env` (`GOOGLE_YOUTUBE_CLIENT_ID/SECRET`, `GOOGLE_YOUTUBE_LOGIN_HINT`) + Credential Manager tokens.
- AI planner (M6): provider-agnostic (OpenAI / Anthropic / OpenAI-compatible / local Ollama) with deterministic template fallback; keys in Credential Manager.

## Completed — M1 foundation
- [x] Tauri shell + 10-screen navigation, no mock data in frontend
- [x] SQLite migrations 001–004 with pre-migration `VACUUM INTO` backups
- [x] Creator profile, settings, creator-rank recalculation
- [x] Rust tests 18/18; Python M1 tests 14/14
- [x] `npm run typecheck` + `npm run build` pass
- [x] README rewritten honestly; acceptance report rewritten honestly
- [x] Live DB migrated and verified at 005

## Completed — M2 campaign & episode management
- [x] Backend: `cmd_create_campaign`, `cmd_update_campaign`, `cmd_archive_campaign`
- [x] Backend: `cmd_create_episode`, `cmd_update_episode`
- [x] Backend: `cmd_set_task_status` with dependency validation + XP award + dependent unlock
- [x] Backend: `cmd_reopen_task`
- [x] All 7 commands registered in `lib.rs` invoke_handler
- [x] Frontend API wrappers in `lib/commands.ts`
- [x] Campaigns screen: create, edit, archive, refresh, sample badge
- [x] Quest Map: start/pause/resume/submit-for-review/approve/complete/reopen per task, dependency gating
- [x] Daily Quests: lists available/in-progress tasks for active episode with start/pause
- [x] M2 Rust tests: campaign/episode creation, dependency unlock chain, dependency-gated completion

## Completed — M3 gamification detail
- [x] Migration `005_achievement_notifications.sql`: `achievement_notifications` table + dedupe keys + CHARACTER_CREATOR fix
- [x] Backend: `evaluate_achievements()` — task_completed, stage_status (all 15 canonical stages), episode_completed, published_count, channel_views, subscribers, watch_minutes, consistency_weeks; manual never auto-unlocks; hidden achievements still evaluated
- [x] Backend: `record_achievement_notifications()` — individual unlock + consolidated backfill summary, durable with dedupe
- [x] Backend: `record_productive_day()` rewritten — multi-day gap iteration, rest/vacation exclusion, up to 2 eligible missed days bridged with freezes, >2 resets, vacation_until inclusive + expiry
- [x] Backend: `cmd_get_achievement_notifications`, `cmd_acknowledge_achievement_notification`, `cmd_set_streak_prefs`
- [x] Hooks: task completion → evaluate + notify; streak record → evaluate + notify; thresholds → backfill on bootstrap; bootstrap backfill on load
- [x] Frontend: `isHidden` on Achievement, `status` on ActivityDay, new DTOs (AchievementUnlock, AchievementNotification, SetStreakPrefsInput, AcknowledgeNotificationsInput)
- [x] Frontend: API wrappers + store actions (fetchNotifications, acknowledgeNotifications, setStreakPrefs)
- [x] Frontend: ActivityHeatmap component (7-day, status precedence: productive > vacation > rest > missed)
- [x] Frontend: Journey streak detail + heatmap; Dashboard heatmap replaces bar chart
- [x] Frontend: Achievements screen hides hidden locked achievements
- [x] Frontend: Settings streak prefs (rest day, vacation mode, vacation until)
- [x] M3 Rust tests: first-task unlock, multi-user keys, hidden achievement, 3-day gap reset, 2-day freeze consume, rest-day exclusion, notification persist/ack, backfill idempotency, reopened task progress

## Completed — M4 YouTube integration
- [x] OAuth 2.0 Desktop PKCE + loopback `127.0.0.1:0`, `select_account` + `login_hint` (.env `GOOGLE_YOUTUBE_LOGIN_HINT`)
- [x] `.env` secrets (dotenvy) + Credential Manager token storage, never SQLite
- [x] Migration `006_youtube_sync`: channel + analytics snapshots; sync updates `creator_progress` → achievement engine
- [x] Data API `channels.list?mine=true` + Analytics `reports.query` 28d (views/watch/subs gained/lost)
- [x] Frontend: credentials form, Connect/Disconnect/Sync, metrics dashboard
- [x] Live DB migrated to `006_youtube_sync`; live OAuth needs test-user add + Connect + Sync in app (not yet connected in DB)

## Completed — M5 reminders + scheduler + quiet hours
- [x] Reminder CRUD (`create/update/delete/enable/snooze 5-720m`) + validation, each mutation syncs its `StoryQuestStudio\…` schtasks entry (delete removes it)
- [x] schtasks fixes: weekly `/D` from `days_of_week_json` (empty = `*`), specific_date `/SC ONCE /SD <date>`; pure `build_schtasks_args` + 5 unit tests
- [x] Quiet-hours enforcement in test-toast + `--reminder-mode` fire path; suppressed = `notification_history (quiet_hours_suppressed)` row
- [x] `--reminder-mode --reminder-id` loads the real reminder (enabled/snooze/quiet checks, `last_triggered_at`, `delivered` history row)
- [x] Calendar screen CRUD UI (types, time, weekly day picker, date, enable, snooze 30m, delete); Settings quiet-hours = active
- [x] Manual verify: `schtasks /Create /SC WEEKLY /D MON,WED,FRI` created + queried + deleted `StoryQuestStudio\M5Test`

## Completed — M6 AI provider + roadmap generation
- [x] Provider commands: list / set-active / update (base URL, model, temp, tokens) / test-connection; keys via existing Credential Manager commands
- [x] `generate_roadmap` async: OpenAI + OpenAI-compatible (OpenRouter/Groq/Ollama/LM Studio) via `POST {base}/chat/completions` JSON mode, Anthropic via `POST {base}/messages` (`x-api-key`, `anthropic-version`); model JSON hydrated onto canonical 15-stage skeleton; any failure → deterministic template with `source` = `template` (vs `ai`)
- [x] `cmd_apply_generated_roadmap`: inserts tasks + `task_dependencies`, updates episode premise/structure; refuses episodes with existing tasks
- [x] Frontend: new **AI Planner** screen (campaign/episode/premise form → generate → preview → apply) + provider panel (save key, make active, test)
- [x] Unit tests: JSON fence stripping, model-JSON hydration + dependency chaining, invalid-JSON rejection, template fallback source

## In progress
- [ ] M5 live acceptance: create a reminder in Calendar, confirm `schtasks /Query` shows it, fire test toast
- [ ] M6 live acceptance: set a provider key in AI Planner, Test connection, Generate + Apply a roadmap

## Accepted
- [x] User acceptance review (M3) — 2026-11-09
- [x] M4 live YouTube sync — 2026-11-09: connected `createcreate313@gmail.com`, channel "Fantasy HR" (`UCXaZOMvGe_9So3su9yes04g`), 2 videos, 217 views, 0 subs; 28d analytics: 3 views, 0 watch min. Verified in app DB `%LOCALAPPDATA%\com.storyquest.studio\storyquest.db` (GUI app data dir; project `StoryQuestProjects/` DB is migrate-only and stays `not_connected`).

## Next
- [ ] M5 + M6 live acceptance in `tauri dev` (M7 installer build skipped 2026-11-09 — see Later)

## Later (deferred — documented 2026-11-09)
- [ ] **M7 — Packaging & release**: **SKIPPED by user 2026-11-09** (not needed for functionality; M5/M6 validate in `tauri dev`). `npm run tauri:build` (NSIS `.exe` + WiX `.msi` under `src-tauri/target/release/bundle/`), install on clean Windows, smoke-test: first launch, DB migration to 006, YouTube connect, reminder fires while closed, AI Planner template path. `scripts/build-windows.ps1` still matches. Checkpoint: tag `pre-m7-2026-11-09`, DB backups `StoryQuestProjects/Backups/*.20261109-113916.bak`.
- [ ] **Live acceptance (needs user in app)**: M5 — create a Calendar reminder, confirm with `schtasks /Query /TN "StoryQuestStudio\…"`, fire test toast (outside quiet hours). M6 — paste a provider key in AI Planner, Test connection, Generate + Apply a roadmap to a task-less episode, confirm tasks in Quest Map.
- [ ] **YouTube OAuth production**: Google Cloud Console → OAuth consent screen → **Publish app** (leaves Testing; refresh tokens stop expiring after 7 days; unverified-app warning remains for sensitive scopes until Google verification).
- [ ] **Polish / backlog**: `npm audit` vulnerabilities review; silence Rust warnings (unused `streak_freezes_available`, `current_streak`, `mode`, `mut get`, dead `condition_value`); `python` milestone script could assert M4/M5 tables; MCP agent execution (`mcp_integration.rs` is export-only) and asset library remain post-M7.

## Verification log
| Date | Check | Result |
| :--- | :--- | :--- |
| 2026-11-09 | `cargo test` | 18/18 pass (7 M1 + 3 M2 + 8 M3) |
| 2026-11-09 | `python tests/verify_all_milestones.py` | 14/14 pass |
| 2026-11-09 | `npm run typecheck` | pass |
| 2026-11-09 | `npm run build` | pass |
| 2026-11-09 | `cargo check` | pass |
| 2026-11-09 | `cargo test` (M4) | 24/24 pass (5 new YouTube tests) |
| 2026-11-09 | Live DB migration to 006 | verified: `006_youtube_sync`, youtube settings keys present, 0 snapshots (not yet connected) |
| 2026-11-09 | Live DB migration to 005 | verified: notification table, CHARACTER_CREATOR fix |
| 2026-11-09 | `cargo test` (M5) | 29/29 pass (5 new schtasks tests) |
| 2026-11-09 | `python tests/verify_all_milestones.py` (M5) | 14/14 pass |
| 2026-11-09 | `npm run typecheck` + `npm run build` (M5) | pass |
| 2026-11-09 | Manual `schtasks` M5Test | create/query/delete `StoryQuestStudio\M5Test` OK |
| 2026-11-09 | M4 live YouTube sync | verified: app DB connected, Fantasy HR, 2 videos, 217 views, 0 subs |
| 2026-11-09 | `cargo test` (M6) | 33/33 pass (4 new AI planner tests) |
| 2026-11-09 | `python tests/verify_all_milestones.py` (M6) | 14/14 pass |
| 2026-11-09 | `npm run typecheck` + `npm run build` (M6) | pass |
| 2026-11-09 | M7 checkpoint + pre-flight | tag `pre-m7-2026-11-09`; 2x DB backups 327KB; python 14/14, typecheck/build pass, cargo 33/33 (7 warnings deferred) |
| 2026-11-09 | M7 bundler prereqs | WiX v3 + NSIS missing; MSVC BuildTools present; build skipped by user decision |
| 2026-11-09 | Fix `input` arg wrapping (16 commands) | `src/lib/commands.ts`: `call(cmd, input)` → `call(cmd, {input})` (+ ApplyRoadmap `{input:{roadmap}}`); typecheck/build pass, cargo 33/33 |
| 2026-11-09 | Fix `cmd_set_streak_prefs` upsert | `ON CONFLICT(user_id)` invalid (no UNIQUE on `user_id`, PK is `id`); UPDATE-then-INSERT; cargo 33/33 |
