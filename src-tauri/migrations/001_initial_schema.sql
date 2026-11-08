-- StoryQuest Studio: Migration 001 - Complete Core Schema (21 Core Tables + Skills & Backups)
-- Connection pragmas are applied by the migration runner, never inside a transaction.

-- 1. Users
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL DEFAULT 'Solo Creator',
    studio_name TEXT NOT NULL DEFAULT 'StoryQuest Animation Studio',
    current_level INTEGER NOT NULL DEFAULT 1,
    total_xp INTEGER NOT NULL DEFAULT 0,
    active_title TEXT NOT NULL DEFAULT 'Apprentice',
    active_theme TEXT NOT NULL DEFAULT 'obsidian_gold',
    avatar_badge TEXT NOT NULL DEFAULT 'crest_quill',
    daily_xp_custom_tasks INTEGER NOT NULL DEFAULT 0,
    last_custom_xp_date TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 2. Settings
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'general',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 3. Campaigns (Series Projects)
CREATE TABLE IF NOT EXISTS campaigns (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    synopsis TEXT NOT NULL,
    genre TEXT NOT NULL DEFAULT 'Dark Fantasy',
    visual_style TEXT NOT NULL DEFAULT '2.5D Stylized Cel-Shaded',
    world_info TEXT NOT NULL DEFAULT '',
    target_runtime_minutes INTEGER NOT NULL DEFAULT 8,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('planning', 'active', 'paused', 'completed', 'archived')),
    project_folder_path TEXT NOT NULL,
    progress_percent REAL NOT NULL DEFAULT 0.0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 4. Episodes
CREATE TABLE IF NOT EXISTS episodes (
    id TEXT PRIMARY KEY,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    episode_number INTEGER NOT NULL,
    title TEXT NOT NULL,
    premise TEXT NOT NULL DEFAULT '',
    narrative_structure TEXT NOT NULL DEFAULT '',
    target_duration_sec INTEGER NOT NULL DEFAULT 480,
    current_stage INTEGER NOT NULL DEFAULT 1 CHECK (current_stage BETWEEN 1 AND 15),
    roadmap_approved INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'in_progress' CHECK (status IN ('draft', 'in_progress', 'review', 'completed', 'published')),
    youtube_url TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(campaign_id, episode_number)
);

-- 5. Tasks (Quests)
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    episode_id TEXT NOT NULL REFERENCES episodes(id) ON DELETE CASCADE,
    stage_number INTEGER NOT NULL CHECK (stage_number BETWEEN 1 AND 15),
    stage_name TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    objective TEXT NOT NULL,
    required_input TEXT NOT NULL DEFAULT '',
    ai_instructions TEXT NOT NULL DEFAULT '',
    human_responsibilities TEXT NOT NULL DEFAULT '',
    required_software TEXT NOT NULL DEFAULT 'Blender / Moho / Markdown',
    expected_output TEXT NOT NULL DEFAULT '',
    estimated_effort_mins INTEGER NOT NULL DEFAULT 30,
    actual_effort_mins INTEGER NOT NULL DEFAULT 0,
    definition_of_done TEXT NOT NULL DEFAULT 'Deliverable saved and verified in project folder.',
    completion_requirement_type TEXT NOT NULL DEFAULT 'confirmation' CHECK (completion_requirement_type IN ('artifact', 'file', 'note', 'confirmation')),
    completion_artifact_value TEXT NOT NULL DEFAULT '',
    difficulty TEXT NOT NULL DEFAULT 'medium' CHECK (difficulty IN ('small', 'medium', 'large', 'milestone')),
    xp_reward INTEGER NOT NULL DEFAULT 25,
    skill_category TEXT NOT NULL DEFAULT 'Storytelling',
    status TEXT NOT NULL DEFAULT 'locked' CHECK (status IN ('locked', 'blocked', 'available', 'in_progress', 'paused', 'review', 'approved', 'completed')),
    is_daily_quest INTEGER NOT NULL DEFAULT 0,
    due_date TEXT,
    notes TEXT NOT NULL DEFAULT '',
    attached_files_json TEXT NOT NULL DEFAULT '[]',
    completed_at TEXT,
    approved_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 6. Task Dependencies
CREATE TABLE IF NOT EXISTS task_dependencies (
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    depends_on_task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    require_approval INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (task_id, depends_on_task_id),
    CHECK (task_id != depends_on_task_id)
);

-- 7. Milestones
CREATE TABLE IF NOT EXISTS milestones (
    id TEXT PRIMARY KEY,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    episode_id TEXT REFERENCES episodes(id) ON DELETE CASCADE,
    stage_number INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    xp_reward INTEGER NOT NULL DEFAULT 100,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'ready_for_review', 'approved', 'completed')),
    due_date TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 8. XP Events (Idempotent Anti-Exploit Ledger)
CREATE TABLE IF NOT EXISTS xp_events (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    idempotency_key TEXT NOT NULL UNIQUE,
    source_type TEXT NOT NULL CHECK (source_type IN ('task_completion', 'milestone_approval', 'episode_completion', 'season_completion', 'achievement_unlock', 'focus_bonus')),
    source_id TEXT NOT NULL,
    xp_amount INTEGER NOT NULL CHECK (xp_amount > 0),
    skill_category TEXT,
    description TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 9. Levels
CREATE TABLE IF NOT EXISTS levels (
    level INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    min_xp INTEGER NOT NULL,
    badge_icon TEXT NOT NULL,
    perks_description TEXT NOT NULL
);

-- 10. Achievements
CREATE TABLE IF NOT EXISTS achievements (
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    category TEXT NOT NULL,
    xp_bonus INTEGER NOT NULL DEFAULT 50,
    badge_svg_key TEXT NOT NULL,
    cosmetic_reward TEXT NOT NULL DEFAULT '',
    unlocked INTEGER NOT NULL DEFAULT 0,
    unlocked_at TEXT,
    progress_current INTEGER NOT NULL DEFAULT 0,
    progress_target INTEGER NOT NULL DEFAULT 1
);

-- 11. Streak Records
CREATE TABLE IF NOT EXISTS streak_records (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    current_streak INTEGER NOT NULL DEFAULT 0,
    best_streak INTEGER NOT NULL DEFAULT 0,
    streak_freezes_available INTEGER NOT NULL DEFAULT 2,
    weekly_rest_day INTEGER NOT NULL DEFAULT 0 CHECK (weekly_rest_day BETWEEN 0 AND 6),
    vacation_mode INTEGER NOT NULL DEFAULT 0,
    vacation_until TEXT,
    last_productive_date TEXT,
    history_json TEXT NOT NULL DEFAULT '[]',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 12. Focus Sessions
CREATE TABLE IF NOT EXISTS focus_sessions (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    planned_duration_mins INTEGER NOT NULL,
    elapsed_seconds INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'paused', 'completed', 'partial', 'rescheduled', 'abandoned')),
    quick_notes TEXT NOT NULL DEFAULT '',
    outcome_summary TEXT NOT NULL DEFAULT '',
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at TEXT
);

-- 13. Reminders
CREATE TABLE IF NOT EXISTS reminders (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    reminder_type TEXT NOT NULL CHECK (reminder_type IN ('daily', 'weekly', 'specific_date', 'task_deadline', 'custom', 'weekly_review')),
    task_id TEXT REFERENCES tasks(id) ON DELETE SET NULL,
    schedule_time TEXT NOT NULL DEFAULT '09:00',
    days_of_week_json TEXT NOT NULL DEFAULT '[1,2,3,4,5]',
    specific_date TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    snoozed_until TEXT,
    windows_task_name TEXT NOT NULL UNIQUE,
    last_triggered_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 14. Notification History
CREATE TABLE IF NOT EXISTS notification_history (
    id TEXT PRIMARY KEY,
    reminder_id TEXT REFERENCES reminders(id) ON DELETE SET NULL,
    task_id TEXT REFERENCES tasks(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    delivery_status TEXT NOT NULL CHECK (delivery_status IN ('delivered', 'missed_recovered', 'snoozed', 'skipped', 'quiet_hours_suppressed', 'permission_failed')),
    user_action TEXT,
    triggered_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 15. Assets (Workshop General & Linked Files)
CREATE TABLE IF NOT EXISTS assets (
    id TEXT PRIMARY KEY,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    episode_id TEXT REFERENCES episodes(id) ON DELETE SET NULL,
    task_id TEXT REFERENCES tasks(id) ON DELETE SET NULL,
    workshop_section TEXT NOT NULL CHECK (workshop_section IN (
        'story_writing', 'scripts', 'character_profiles', 'character_designs',
        'rig_info', 'animation_clips', 'background_assets', 'props',
        'storyboards', 'audio_files', 'scene_production'
    )),
    name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_type TEXT NOT NULL DEFAULT 'markdown',
    version TEXT NOT NULL DEFAULT 'v1.0',
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'in_review', 'approved', 'final')),
    metadata_json TEXT NOT NULL DEFAULT '{}',
    preview_summary TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 16. Characters
CREATE TABLE IF NOT EXISTS characters (
    id TEXT PRIMARY KEY,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'Protagonist',
    archetype TEXT NOT NULL DEFAULT '',
    biography TEXT NOT NULL DEFAULT '',
    voice_profile TEXT NOT NULL DEFAULT '',
    visual_prompt TEXT NOT NULL DEFAULT '',
    rig_status TEXT NOT NULL DEFAULT 'unrigged' CHECK (rig_status IN ('unrigged', 'design_approved', 'rigging_in_progress', 'rig_tested', 'production_ready')),
    rig_file_path TEXT NOT NULL DEFAULT '',
    color_palette_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 17. Environments
CREATE TABLE IF NOT EXISTS environments (
    id TEXT PRIMARY KEY,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    lighting_mood TEXT NOT NULL DEFAULT 'Cinematic Twilight',
    description TEXT NOT NULL DEFAULT '',
    asset_folder_path TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'concept' CHECK (status IN ('concept', 'layout', 'textured', 'lit_ready')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 18. Animation Libraries
CREATE TABLE IF NOT EXISTS animation_libraries (
    id TEXT PRIMARY KEY,
    campaign_id TEXT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    character_id TEXT REFERENCES characters(id) ON DELETE SET NULL,
    clip_name TEXT NOT NULL,
    action_type TEXT NOT NULL DEFAULT 'Locomotion',
    frame_count INTEGER NOT NULL DEFAULT 24,
    fps INTEGER NOT NULL DEFAULT 24,
    software_used TEXT NOT NULL DEFAULT 'Moho Pro / Blender',
    file_path TEXT NOT NULL DEFAULT '',
    reusable INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 19. AI Provider Configurations (API keys stored in Windows Credential Manager, never in SQLite!)
CREATE TABLE IF NOT EXISTS ai_provider_configs (
    id TEXT PRIMARY KEY,
    provider_name TEXT NOT NULL UNIQUE CHECK (provider_name IN ('openai', 'anthropic', 'openai_compatible', 'local_ollama')),
    display_name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    default_model TEXT NOT NULL,
    credential_key_name TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 0,
    temperature REAL NOT NULL DEFAULT 0.7,
    max_tokens INTEGER NOT NULL DEFAULT 4096,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 20. Agent Connections (MCP & External Software)
CREATE TABLE IF NOT EXISTS agent_connections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    connection_type TEXT NOT NULL CHECK (connection_type IN ('mcp_server', 'coding_agent', 'dcc_software')),
    executable_path TEXT NOT NULL DEFAULT '',
    mcp_endpoint TEXT NOT NULL DEFAULT '',
    protocol_version TEXT NOT NULL DEFAULT '2024-11-05',
    is_configured INTEGER NOT NULL DEFAULT 0,
    requires_approval_for_writes INTEGER NOT NULL DEFAULT 1,
    last_verified_at TEXT,
    status TEXT NOT NULL DEFAULT 'unconfigured' CHECK (status IN ('unconfigured', 'configured_offline', 'connected', 'error'))
);

-- 21. Agent Execution Logs
CREATE TABLE IF NOT EXISTS agent_execution_logs (
    id TEXT PRIMARY KEY,
    agent_connection_id TEXT REFERENCES agent_connections(id) ON DELETE SET NULL,
    task_id TEXT REFERENCES tasks(id) ON DELETE SET NULL,
    action_type TEXT NOT NULL,
    command_payload_json TEXT NOT NULL,
    human_approved INTEGER NOT NULL DEFAULT 1,
    execution_status TEXT NOT NULL CHECK (execution_status IN ('queued', 'dispatched', 'exported_only', 'completed', 'failed', 'cancelled_by_user')),
    output_log TEXT NOT NULL DEFAULT '',
    executed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Auxiliary Skill Tree Progress Table
CREATE TABLE IF NOT EXISTS skill_categories (
    category TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    xp_earned INTEGER NOT NULL DEFAULT 0,
    rank_level INTEGER NOT NULL DEFAULT 1,
    tasks_completed INTEGER NOT NULL DEFAULT 0,
    icon_key TEXT NOT NULL,
    description TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tasks_episode_stage ON tasks(episode_id, stage_number);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_xp_events_user ON xp_events(user_id);
CREATE INDEX IF NOT EXISTS idx_reminders_enabled ON reminders(enabled, schedule_time);
