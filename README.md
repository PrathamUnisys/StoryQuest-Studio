# StoryQuest Studio

> **"Every episode is a quest. Every finished story is an achievement."**

**StoryQuest Studio** is a local-first, gamified animated-production manager for **Windows 10/11**, built with **Tauri 2 (Rust + SQLite)** and **React + TypeScript + Vite + Tailwind CSS + Zustand**.

Designed for solo animators and independent YouTube creators, it tracks creator progression ranked on **published videos**, manages animation campaigns/episodes/tasks, syncs real YouTube channel stats, and fires Windows scheduled reminders — with an honest, real-data-only interface. It is **not** a static demo: it reads and writes a persistent SQLite database, saves settings across restarts, and blocks browser preview so no sample statistics are ever shown as real.

---

## Status — M6 complete (accepted: M3, M4 live)

| Area | State |
| :--- | :--- |
| Tauri shell + 10-screen navigation | Implemented |
| SQLite migrations `001`–`006` + backups | Implemented |
| Creator profile / settings / creator ranks | Implemented |
| Campaign/episode/task CRUD + dependency unlocks | Implemented (M2) |
| Achievement auto-unlock + notifications + streaks | Implemented (M3) |
| YouTube OAuth (PKCE + loopback) + Data/Analytics sync | Implemented + live-verified (M4) |
| Reminder CRUD + Windows Task Scheduler + quiet hours | Implemented (M5) |
| AI provider settings + roadmap generation (OpenAI / Anthropic / compatible / Ollama) | Implemented (M6) |
| Packaging / installer smoke test | **M7** — not yet (see Later in `TASK.md`) |

See `TASK.md` for the current working state and `docs/MILESTONE_ACCEPTANCE_REPORT.md` for the honest report.

---

## Technology stack

| Component | Technology | Version |
| :--- | :--- | :--- |
| Desktop framework | Tauri 2 (Rust) | 2.12.5 |
| Frontend | React + TypeScript + Vite | React 18.3 |
| UI components | shadcn/ui style (Radix) + Tailwind CSS | Tailwind 3.4 |
| State | Zustand | 5.1.0 |
| Charts | Recharts | 3.10.1 |
| Local database | SQLite (`rusqlite` bundled) | 0.32.1 |
| Credentials / OAuth tokens | Windows Credential Manager (`keyring`) + `.env` (dotenvy) | 3.6.1 / 0.15 |
| YouTube APIs | Data API v3 + Analytics API via OAuth 2.0 PKCE | — |
| Notifications | Windows native toasts (`winrt-notification`) | 0.5.1 |
| Reminders | Windows Task Scheduler (`schtasks.exe`) | — |
| Startup | Tauri Autostart | 2.7.0 |

---

## Directory structure

```
storyquest-studio/
├── package.json                      # Scripts + frontend deps
├── tsconfig.json, vite.config.ts
├── tailwind.config.js, postcss.config.js
├── index.html
├── .env.example                      # Copy to .env — Google OAuth credentials (never commit .env)
├── TASK.md                           # Where-we-are tracker (this project)
├── src/                              # React + TypeScript frontend
│   ├── main.tsx                      # React root mount
│   ├── App.tsx                       # Shell + 10-screen router + Tauri guard
│   ├── types/storyquest.ts           # Shared DTOs
│   ├── lib/                          # commands.ts, tauriBridge.ts, navigation.ts, format.ts, utils.ts
│   ├── store/useStoryQuestStore.ts   # Real Zustand store, no mock data
│   ├── screens/*                     # 10 screens (dashboard → settings)
│   ├── components/ui/*               # shadcn-style primitives
│   ├── components/layout/*           # App shell, sidebar, top bar
│   └── components/common/*           # Empty/error/loading/notice/stat-card/page-header
├── src-tauri/                        # Rust backend
│   ├── Cargo.toml                    # Deps: tauri 2.12.5, rusqlite 0.32.1, reqwest, keyring, sha2/base64/dotenvy
│   ├── tauri.conf.json               # Window, tray, CSP, autostart
│   ├── migrations/
│   │   ├── 001_initial_schema.sql        # 22 core tables + settings + levels + 9 achievements
│   │   ├── 002_seed_data.sql             # Demo levels, skills, AI provider configs, agent connections
│   │   ├── 003_foundation.sql            # creator_ranks (13), creator_progress, 24 achievements,
│   │   │                                 # resets fabricated progression, sample campaign camp_01 + 15 tasks
│   │   ├── 004_repair_achievement_seed.sql # Idempotent repair for ach_10–ach_24 seed bug
│   │   ├── 005_achievement_notifications.sql # Durable achievement notifications + streak controls
│   │   └── 006_youtube_sync.sql          # YouTube channel + analytics snapshots (M4)
│   └── src/
│       ├── main.rs / lib.rs            # Tauri builder, CLI --migrate-only, tray, --reminder-mode
│       ├── commands.rs                 # Bootstrap + campaign/episode/task + reminders + YouTube + settings
│       ├── models.rs                   # DTOs (camelCase), PRODUCTION_STAGES 1–15
│       ├── db.rs                       # Migration runner + VACUUM INTO backups
│       ├── gamification.rs             # Idempotent XP ledger, creator-rank recalculation, achievements
│       ├── youtube.rs                  # OAuth PKCE/loopback, token store, Data/Analytics sync (M4)
│       ├── scheduler.rs, tray.rs, windows_task_scheduler.rs  # Toasts, quiet hours, schtasks (M5)
│       ├── ai_planner.rs               # Deterministic offline template (M6 later)
│       ├── mcp_integration.rs          # Export-only safe integration (M6 later)
│       ├── credentials.rs              # Windows Credential Manager wrapper
│       ├── project_fs.rs               # Campaign folder scaffolding 00_Management→12_YouTube
│       └── tests.rs                    # 24+ Rust unit tests (M1–M5)
├── scripts/
│   └── build-windows.ps1             # Build/packaging script
├── tests/
│   └── verify_all_milestones.py      # Foundation verification (14 tests, temp DB only)
├── docs/
│   ├── MILESTONE_ACCEPTANCE_REPORT.md # Honest per-milestone report
│   ├── DATABASE_DOCUMENTATION.md
│   ├── AI_PROVIDER_GUIDE.md
│   ├── MCP_INTEGRATION_GUIDE.md
│   └── REMINDER_TESTING_GUIDE.md
└── StoryQuestProjects/
    └── storyquest.db                   # Local SQLite DB (not versioned; migrate-only target)
```

> **Note:** the running GUI app stores its DB under `%LOCALAPPDATA%\com.storyquest.studio\storyquest.db`
> (Tauri `app_local_data_dir`); `StoryQuestProjects/storyquest.db` is the `--migrate-only` target.

---

## Database (M1–M5)

- 24 application tables + `schema_migrations` tracking table.
- `creator_ranks`: 13 ranks, thresholds on **published video count** only: 0 / 1 / 3 / 5 / 10 / 15 / 25 / 40 / 60 / 100 / 150 / 250 / 500.
- `creator_progress`: rank level, published videos, shorts vs long-form counters, subscriber count, total channel views, total watch minutes.
- 24 achievements, all locked by default; auto-unlock engine (M3) evaluates task/stage/episode/published/views/subscribers/watch/consistency conditions, with durable deduped notifications.
- `camp_01` "Chronicles of Aethelgard" is a clearly labeled sample (`is_sample = 1`), planning stage, 15 tasks (1 available / 14 locked).
- `youtube_channel_snapshots` / `youtube_analytics_snapshots` (M4): every sync appends a row; latest row drives dashboard + rank.
- `reminders` + `notification_history` (M5): CRUD-backed reminders; quiet-hours suppressions, snoozes, deliveries and missed-recovery summaries are all logged.
- All fabricated progression (Aria Vance, Level 3, 285 XP, 5-day streak) was reset to Solo Creator / Level 1 / 0 XP / 0 streak.

Migrations never lose existing data; each migration creates a `VACUUM INTO` backup under `StoryQuestProjects/Backups/` before applying.

---

## YouTube setup (M4)

1. **Google Cloud Console** → create/select a project.
2. **API Library** → enable **YouTube Data API v3** + **YouTube Analytics API**.
3. **OAuth consent screen** → **External**, add your channel Google account under **Test users** (required while Testing).
4. **Credentials → OAuth client ID → Desktop app** → copy **Client ID** + **Client Secret**.
5. Copy `.env.example` to `.env` and fill in:
   ```
   GOOGLE_YOUTUBE_CLIENT_ID=…apps.googleusercontent.com
   GOOGLE_YOUTUBE_CLIENT_SECRET=…
   GOOGLE_YOUTUBE_LOGIN_HINT=your-channel@gmail.com   # optional: pre-selects the account
   ```
   (`.env` is gitignored; tokens are stored in Windows Credential Manager — never in SQLite.)
6. In the app: **YouTube → Connect YouTube** (account chooser appears) → **Sync now**.
   Scopes: `youtube.readonly`, `yt-analytics.readonly`, `yt-analytics-monetary.readonly`.
   Thumbnail impressions / CTR are not exposed by the Analytics API and stay unavailable — never inferred.
   Live-verified channel: "Fantasy HR" — 2 videos, 217 views, 0 subscribers (2026-11-09).

---

## Reminders (M5)

- **Calendar screen** → New → daily / weekly (day picker) / specific-date reminders.
- Each enabled reminder registers a real Windows task `StoryQuestStudio\Reminder_…` running
  `storyquest-studio.exe --reminder-mode --reminder-id <id>` — it fires even when the app is closed.
- **Quiet hours** (Settings, default 22:30–07:30) suppress delivery and log a `quiet_hours_suppressed` row.
- **Snooze 30m** / enable / disable / delete are all wired; deleting a reminder removes its OS task.
- Missed runs (PC off) recover into a single summary on next launch.

---

## Build & run (Windows)

### Prerequisites
- Windows 10 (1809+) / 11 with WebView2 Runtime.
- Rust toolchain (MSVC) — verify with `cargo --version`.
- Node.js 22 LTS / npm — verify with `node --version && npm --version`.

### Development
```powershell
# 1. Install frontend dependencies
npm install

# 2. Run verification suite (temp DB only, never touches live DB)
python tests/verify_all_milestones.py

# 3. Typecheck + build frontend
npm run build

# 4. Launch Tauri dev app
npm run tauri:dev
```

### Production installer
```powershell
powershell -ExecutionPolicy Bypass -File scripts/build-windows.ps1
```
Produces NSIS `.exe` and WiX `.msi` under `src-tauri/target/release/bundle/`.

---

## Known limitations (honest)

1. **AI roadmap needs a provider key for AI output** (AI Planner screen → save key → make active → test). Without one, the offline template is used and labeled as such — never presented as AI.
2. **Notifications do not fire while the PC is powered off** (S4/S5); missed reminders are recovered into a single summary on next boot.
3. **YouTube OAuth app is in Testing mode** until published in Google Cloud Console; test users only, refresh tokens expire after 7 days in Testing.
4. **`npm audit` reports vulnerabilities** — determine production impact before upgrading.

---

## Roadmap

M7 Packaging & Release (installer smoke test) → deferred backlog (live acceptances, OAuth publish, audit/warnings, MCP execution). See `TASK.md` → "Later (deferred)" for the full list.
