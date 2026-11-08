# StoryQuest Studio — Milestone Acceptance Report

**Generated:** 2026-11-09  
**Current milestone:** M3 — Gamification detail (pending acceptance)  
**Rule applied:** Never claim a feature complete without testing it; never fabricate data.

---

## Milestone 1 — Desktop Foundation

**Status:** Implemented and accepted.

### Features implemented
| Feature | Evidence |
| :--- | :--- |
| Tauri 2 shell, WebView2 window, single instance | `src-tauri/src/lib.rs:72-86`; `tauri.conf.json` |
| 10-screen navigation (Dashboard → Settings) | `src/App.tsx:39-64`; `src/lib/navigation.ts` |
| SQLite migrations `001`–`004` with `schema_migrations` tracking | `src-tauri/src/db.rs:13-30` |
| Pre-migration `VACUUM INTO` backups under `Backups/` | `src-tauri/src/db.rs:79-112` |
| Creator profile + persistent settings | `src-tauri/src/commands.rs`; `src/store/useStoryQuestStore.ts` |
| Creator ranks (13) based on published-video count only | `migrations/003_foundation.sql`; `src-tauri/src/gamification.rs:296-377` |
| XP ledger with unique idempotency key (no duplicate awards) | `src-tauri/src/gamification.rs:22-152` |
| Fabricated progression reset (Solo Creator / L1 / 0 XP / 0 streak) | `migrations/003_foundation.sql` |
| Sample campaign `camp_01` + `ep_01` + 15 tasks, flagged `is_sample=1` | `migrations/003_foundation.sql` |
| Backup / restore command | `src-tauri/src/commands.rs` |
| App info / data folder / autostart plumbing | `src-tauri/src/commands.rs`, `lib.rs` |

### Tests executed (M1)
- `cargo test` — **7/7 pass**
- `python tests/verify_all_milestones.py` — **11/11 pass**

---

## Milestone 2 — Campaign & Episode Management

**Status:** Implemented and verified.

### Features implemented
| Feature | Evidence |
| :--- | :--- |
| Create campaign (title, synopsis, genre, visual style, runtime, optional folder scaffolding) | `src-tauri/src/commands.rs:cmd_create_campaign` |
| Update campaign fields | `cmd_update_campaign` |
| Archive campaign (status → archived) | `cmd_archive_campaign` |
| Create episode (campaign, title, premise, duration, auto episode number) | `cmd_create_episode` |
| Update episode fields incl. roadmap approval + YouTube URL | `cmd_update_episode` |
| Task status transitions with dependency validation | `cmd_set_task_status` |
| Block completion when mandatory dependencies unsatisfied | `deps_satisfied` in `commands.rs` |
| Award XP on completion (idempotent ledger) + record productive day | `gamification.rs:award_xp_idempotent` |
| Unlock dependent tasks on completion | `recalculate_task_dependencies` |
| Reopen completed task → available | `cmd_reopen_task` |
| Campaigns UI: create/edit/archive/refresh | `src/screens/campaigns-screen.tsx` |
| Quest Map UI: start/pause/resume/submit/approve/complete/reopen per task | `src/screens/quest-map-screen.tsx` |
| Daily Quests UI: today's available/in-progress tasks | `src/screens/daily-quests-screen.tsx` |

### Tests executed (M2)
- `cargo test` — **10/10 pass** (7 M1 + 3 M2)
- `npm run typecheck` — pass
- `npm run build` — pass
- Quest Map full task flow verified: Start → Pause → Resume → Submit for review → Approve → Complete → Reopen

---

## Milestone 3 — Gamification detail

**Status:** Implemented and verified.

### Features implemented
| Feature | Evidence |
| :--- | :--- |
| Migration `005_gamification_notifications`: `achievement_notifications` table, dedupe keys, CHARACTER_CREATOR fix | `migrations/005_achievement_notifications.sql`; `src-tauri/src/db.rs` |
| Achievement auto-unlock evaluator: `task_completed`, `stage_status` (all 15 canonical stages), `episode_completed`, `published_count`, `channel_views`, `subscribers`, `watch_minutes`, `consistency_weeks` | `src-tauri/src/gamification.rs:evaluate_achievements` |
| Manual achievements never auto-unlock; hidden achievements still evaluated | `gamification.rs:evaluate_achievements` |
| Durable notifications: individual unlock + consolidated backfill summary, dedupe key prevents duplicates | `gamification.rs:record_achievement_notifications` |
| Streak logic: multi-day gap iteration, rest/vacation exclusion, up to 2 eligible missed days bridged with freezes, >2 resets, `vacation_until` inclusive + expiry | `gamification.rs:record_productive_day` |
| Commands: `cmd_get_achievement_notifications`, `cmd_acknowledge_achievement_notification`, `cmd_set_streak_prefs` | `src-tauri/src/commands.rs` |
| Hooks: task completion → evaluate + notify; streak record → evaluate + notify; bootstrap backfill on load | `commands.rs` |
| Frontend: `isHidden` on Achievement, `status` on ActivityDay | `src/types/storyquest.ts`; `src-tauri/src/models.rs` |
| Frontend: ActivityHeatmap (7-day, status precedence: productive > vacation > rest > missed) | `src/components/common/activity-heatmap.tsx` |
| Frontend: Journey streak detail + heatmap; Dashboard heatmap replaces bar chart | `src/screens/journey-screen.tsx`; `src/screens/dashboard-screen.tsx` |
| Frontend: Achievements screen hides hidden locked achievements | `src/screens/achievements-screen.tsx` |
| Frontend: Settings streak prefs (rest day, vacation mode, vacation until) | `src/screens/settings-screen.tsx` |

### Files created or modified (M3)
- `src-tauri/migrations/005_achievement_notifications.sql` — notification table + CHARACTER_CREATOR fix
- `src-tauri/src/db.rs` — register migration 005
- `src-tauri/src/gamification.rs` — achievement evaluator, streak rewrite, notification recorder
- `src-tauri/src/models.rs` — `is_hidden`, `status`, new DTOs
- `src-tauri/src/commands.rs` — notification/ack/streak-prefs commands, evaluator hooks, weekly_activity user filter + local date
- `src-tauri/src/lib.rs` — 3 new command registrations
- `src-tauri/src/tests.rs` — 8 new M3 tests
- `src/types/storyquest.ts` — new DTOs
- `src/lib/commands.ts` — API wrappers
- `src/store/useStoryQuestStore.ts` — notification/streak-prefs actions
- `src/components/common/activity-heatmap.tsx` — new heatmap component
- `src/screens/dashboard-screen.tsx` — heatmap replaces bar chart
- `src/screens/journey-screen.tsx` — streak detail + heatmap
- `src/screens/achievements-screen.tsx` — hidden achievement filtering
- `src/screens/settings-screen.tsx` — streak prefs UI
- `tests/verify_all_milestones.py` — migration 005 + notification/CHARACTER_CREATOR tests

### Tests executed (M3)
- `cargo test` — **18/18 pass** (7 M1 + 3 M2 + 8 M3)
- `python tests/verify_all_milestones.py` — **14/14 pass**
- `npm run typecheck` — pass
- `npm run build` — pass
- `cargo check` — pass

### Outstanding issues (tracked in TASK.md)
1. `scripts/build-windows.ps1` lacks `$LASTEXITCODE` checks; step numbering inconsistent.
2. `npm audit` reports 2 vulnerabilities (moderate + high).
3. GUI smoke test against live DB not yet completed.
4. YouTube-backed achievement conditions (`published_count`, `channel_views`, `subscribers`, `watch_minutes`) stay at 0 until M4 credentials are available — honest, not fabricated.

### Next milestone
M4 — YouTube Integration (Google OAuth 2.0, YouTube Data/Analytics APIs, channel dashboard). Requires your Google Cloud OAuth client credentials. Begin only after M3 acceptance.
