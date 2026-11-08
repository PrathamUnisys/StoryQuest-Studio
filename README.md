# StoryQuest Studio

> **"Every episode is a quest. Every finished story is an achievement."**

**StoryQuest Studio** is a local-first, gamified animated-production manager for **Windows 10/11**, built with **Tauri 2 (Rust + SQLite)** and **React + TypeScript + Vite + Tailwind CSS + Zustand**.

Designed for solo animators and independent YouTube creators, it tracks creator progression ranked on **published videos**, manages animation campaigns/episodes/tasks, and provides an honest, real-data-only interface. It is **not** a static demo: it reads and writes a persistent SQLite database, saves settings across restarts, and blocks browser preview so no sample statistics are ever shown as real.

---

## Status — Milestone 1: Desktop Foundation (pending acceptance)

| Area | State |
| :--- | :--- |
| Tauri shell + 10-screen navigation | Implemented |
| SQLite migrations `001`–`004` + backups | Implemented |
| Creator profile / settings / creator ranks | Implemented |
| Live dashboard (no mock data) | Implemented |
| System tray / autostart plumbing | Wired (UI visible) |
| Campaign CRUD / task status transitions | **M2** — not yet |
| Task Scheduler reminder configuration | **M5** — unwired |
| Achievement auto-unlock | **M3** — returns empty |
| YouTube OAuth / real channel stats | **M4** — no credentials yet |
| AI production planner | **M6** — deterministic template only |

See `docs/MILESTONE_ACCEPTANCE_REPORT.md` for the full honest report.

---

## Technology stack (§14)

| Component | Technology | Version |
| :--- | :--- | :--- |
| Desktop framework | Tauri 2 (Rust) | 2.12.5 |
| Frontend | React + TypeScript + Vite | React 18.3 |
| UI components | shadcn/ui style (Radix) + Tailwind CSS | Tailwind 3.4 |
| State | Zustand | 5.1.0 |
| Charts | Recharts | 3.10.1 |
| Local database | SQLite (`rusqlite` bundled) | 0.32.1 |
| Credentials | Windows Credential Manager (`keyring`) | 3.6.1 |
| Notifications | Windows native notifications | tauri-plugin-notification 2.8.0 |
| Startup | Tauri Autostart | 2.7.0 |

---

## Directory structure

```
storyquest-studio/
├── package.json                      # Scripts + frontend deps
├── tsconfig.json, vite.config.ts
├── tailwind.config.js, postcss.config.js
├── index.html
├── TASK.md                           # Where-we-are tracker (this project)
├── src/                              # React + TypeScript frontend
│   ├── main.tsx                      # React root mount
│   ├── App.tsx                       # Shell + 10-screen router + Tauri guard
│   ├── types/storyquest.ts           # Shared DTOs
│   ├── lib/                          # commands.ts, tauriBridge.ts, navigation.ts, format.ts, utils.ts
│   ├── store/useStoryQuestStore.ts   # Real Zustand store, no mock data
│   ├── screens/*                     # 10 M1 screens (dashboard → settings)
│   ├── components/ui/*               # shadcn-style primitives
│   ├── components/layout/*           # App shell, sidebar, top bar
│   └── components/common/*           # Empty/error/loading/notice/stat-card/page-header
├── src-tauri/                        # Rust backend
│   ├── Cargo.toml                    # Deps: tauri 2.12.5, rusqlite 0.32.1, plugins
│   ├── tauri.conf.json               # Window, tray, CSP, autostart
│   ├── migrations/
│   │   ├── 001_initial_schema.sql        # 22 core tables + settings + levels + 9 achievements
│   │   ├── 002_seed_data.sql             # Demo levels, skills, AI provider configs, agent connections
│   │   ├── 003_foundation.sql            # creator_ranks (13), creator_progress, 24 achievements,
│   │   │                                 # resets fabricated progression, sample campaign camp_01 + 15 tasks
│   │   └── 004_repair_achievement_seed.sql # Idempotent repair for ach_10–ach_24 seed bug
│   └── src/
│       ├── main.rs / lib.rs            # Tauri builder, CLI --migrate-only, tray, 26 commands
│       ├── commands.rs                 # Bootstrap, profile, settings, ranks, backup, autostart
│       ├── models.rs                   # DTOs (camelCase), PRODUCTION_STAGES 1–15
│       ├── db.rs                       # Migration runner + VACUUM INTO backups
│       ├── gamification.rs             # Idempotent XP ledger, creator-rank recalculation
│       ├── scheduler.rs, tray.rs, windows_task_scheduler.rs
│       ├── ai_planner.rs               # Deterministic offline template (M6 later)
│       ├── mcp_integration.rs          # Export-only safe integration (M6 later)
│       ├── credentials.rs              # Windows Credential Manager wrapper
│       ├── project_fs.rs               # Campaign folder scaffolding 00_Management→12_YouTube
│       └── tests.rs                    # 7 Rust unit tests
├── scripts/
│   └── build-windows.ps1             # Build/packaging script
├── tests/
│   └── verify_all_milestones.py      # M1 foundation verification (11 tests, temp DB only)
├── docs/
│   ├── MILESTONE_ACCEPTANCE_REPORT.md # Honest per-milestone report
│   ├── DATABASE_DOCUMENTATION.md
│   ├── AI_PROVIDER_GUIDE.md
│   ├── MCP_INTEGRATION_GUIDE.md
│   └── REMINDER_TESTING_GUIDE.md
└── StoryQuestProjects/
    └── storyquest.db                   # Local SQLite DB (not versioned)
```

---

## Database (M1)

- 24 application tables + `schema_migrations` tracking table.
- `creator_ranks`: 13 ranks, thresholds on **published video count** only: 0 / 1 / 3 / 5 / 10 / 15 / 25 / 40 / 60 / 100 / 150 / 250 / 500.
- `creator_progress`: rank level, published videos, shorts vs long-form counters.
- 24 achievements, all locked by default (`condition_type` / `condition_value` / `progress_available` columns present; auto-unlock engine deferred to M3).
- `camp_01` "Chronicles of Aethelgard" is a clearly labeled sample (`is_sample = 1`), planning stage, 15 tasks (1 available / 14 locked).
- All fabricated progression (Aria Vance, Level 3, 285 XP, 5-day streak) was reset to Solo Creator / Level 1 / 0 XP / 0 streak.

Migrations never lose existing data; each migration creates a `VACUUM INTO` backup under `StoryQuestProjects/Backups/` before applying.

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

# 2. Run M1 verification suite (temp DB only, never touches live DB)
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

1. **No YouTube integration yet.** `youtube_connection_status` shows `not_connected`. Google OAuth 2.0, YouTube Data/Analytics APIs, and sync logic are M4 work and require your Google Cloud OAuth client credentials.
2. **No campaign CRUD yet.** Campaign/episode/task creation, task status transitions (`locked → available → completed`), and daily-quest generation are M2 work.
3. **Achievement auto-unlock not yet implemented.** The Achievement Hall reads the seeded achievements, but unlock conditions are not evaluated until M3.
4. **Windows Task Scheduler is unwired.** `sync_windows_scheduled_task` exists in Rust but has no UI/DB callers yet (M5).
5. **AI planner is a deterministic offline template.** No provider API calls are made yet (M6).
6. **Notifications do not fire while the PC is powered off** (S4/S5); missed reminders are recovered into a single summary on next boot.
7. **`npm audit` reports 2 vulnerabilities** — determine production impact before upgrading.

---

## Roadmap

M2 Campaign & Episode Management → M3 Gamification / Achievements → M4 YouTube Integration → M5 Windows Reminders → M6 AI Production Planner → M7 Packaging & Release. See `TASK.md` for the current working state.
