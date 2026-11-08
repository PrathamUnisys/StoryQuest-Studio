# StoryQuest Studio — Task Tracker

Single source of truth for "where we are." Updated after every work session.

## Current state (M3 — Gamification detail)
**Status:** Implemented; verification passed. Awaiting user acceptance review before M4.

Live database `StoryQuestProjects/storyquest.db` is at migration `005_gamification_notifications`.
- User: `Solo Creator`, Level 1, 0 XP, 0 streak (fabricated Aria Vance data reset).
- Creator ranks: 13, thresholds 0/1/3/5/10/15/25/40/60/100/150/250/500 published videos.
- Achievements: 24 seeded, all locked until conditions are met. Auto-unlock engine active.
- CHARACTER_CREATOR fixed: `progress_available = 0` (manual achievement).
- Tasks: 15 in sample campaign `camp_01`; 1 available, 14 locked.
- YouTube: `not_connected` (M4; no Google OAuth credentials yet).
- AI planner: deterministic template only (M6).

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

## In progress
- [ ] User acceptance review (M3)

## Next — M4 (after acceptance)
- [ ] Google OAuth 2.0, YouTube Data/Analytics sync, channel dashboard (needs OAuth credentials)

## Later — M5–M7
- [ ] M5: Windows Task Scheduler wiring, reminder CRUD, quiet hours enforcement
- [ ] M6: AI provider settings + provider-agnostic roadmap generation
- [ ] M7: `tauri:build` installer smoke test, NSIS/MSI output verification

## Verification log
| Date | Check | Result |
| :--- | :--- | :--- |
| 2026-11-09 | `cargo test` | 18/18 pass (7 M1 + 3 M2 + 8 M3) |
| 2026-11-09 | `python tests/verify_all_milestones.py` | 14/14 pass |
| 2026-11-09 | `npm run typecheck` | pass |
| 2026-11-09 | `npm run build` | pass |
| 2026-11-09 | `cargo check` | pass |
| 2026-11-09 | Live DB migration to 005 | verified: notification table, CHARACTER_CREATOR fix |
