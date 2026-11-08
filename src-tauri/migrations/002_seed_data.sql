-- StoryQuest Studio: Migration 002 - Seed Progression, Achievements, Skills, Settings & Starter Campaign

INSERT OR IGNORE INTO users (id, display_name, studio_name, current_level, total_xp, active_title, active_theme)
VALUES ('user_solo_01', 'Aria Vance', 'Aethelgard Animation Forge', 3, 285, 'Storyteller', 'obsidian_gold');

-- 10 Progressive Levels
INSERT OR IGNORE INTO levels (level, title, min_xp, badge_icon, perks_description) VALUES
(1, 'Apprentice', 0, 'level_1', 'Unlocks core Quest Map and Daily Focus Timer.'),
(2, 'Story Explorer', 100, 'level_2', 'Unlocks AI Story Premise Analyzer and Workshop Notes.'),
(3, 'Storyteller', 250, 'level_3', 'Unlocks Obsidian Gold Studio Theme & Multi-Episode Tracking.'),
(4, 'Creative Architect', 500, 'level_4', 'Unlocks Progressive 15-Stage Roadmap Decomposition.'),
(5, 'Animation Apprentice', 850, 'level_5', 'Unlocks Reusable Character Rig & Action Clip Library.'),
(6, 'Character Animator', 1300, 'level_6', 'Unlocks Emerald Astral Theme & Custom Focus Presets.'),
(7, 'Animation Director', 1900, 'level_7', 'Unlocks Advanced Continuity Inspector & Shot-List Exporter.'),
(8, 'Story Producer', 2600, 'level_8', 'Unlocks Batch MCP Task JSON Export & Pipeline Analytics.'),
(9, 'Studio Creator', 3500, 'level_9', 'Unlocks Royal Celestial Banner & Multi-Season Archiving.'),
(10, 'Master Storyteller', 4600, 'level_10', 'Grandmaster Crest, All Studio Themes & Master Producer Title.');

-- 9 Core Achievements
INSERT OR IGNORE INTO achievements (id, code, title, description, category, xp_bonus, badge_svg_key, cosmetic_reward, unlocked, unlocked_at, progress_current, progress_target) VALUES
('ach_01', 'FIRST_STORY', 'First Story', 'Complete and approve your first episode script in Stage 2.', 'Writing', 50, 'badge_scroll', 'Title: Script Scribe', 1, '2026-11-05T18:20:00Z', 1, 1),
('ach_02', 'VOICE_OF_WORLD', 'Voice of the World', 'Complete your first voice direction and scratch narration pass.', 'Audio', 50, 'badge_mic', 'Theme Accent: Echo Teal', 1, '2026-11-07T19:45:00Z', 1, 1),
('ach_03', 'CHARACTER_CREATOR', 'Character Creator', 'Approve your first complete character turnaround sheet.', 'Design', 50, 'badge_character', 'Badge: Sovereign Stylus', 0, NULL, 0, 1),
('ach_04', 'PUPPET_MASTER', 'Puppet Master', 'Complete and test your first production-ready character rig.', 'Rigging', 75, 'badge_puppet', 'Title: Bone Weaver', 0, NULL, 0, 1),
('ach_05', 'ANIMATOR', 'Animator', 'Complete your first reusable animation clip in the library.', 'Animation', 75, 'badge_keyframe', 'Theme: Arcane Keyframe', 0, NULL, 0, 1),
('ach_06', 'SCENE_BUILDER', 'Scene Builder', 'Render your first complete animated scene with lighting.', 'Production', 100, 'badge_camera', 'Badge: Lens of Aether', 0, NULL, 0, 1),
('ach_07', 'FIRST_EPISODE', 'First Episode', 'Complete all 15 stages for your first animated episode.', 'Milestone', 250, 'badge_crown', 'Title: Episode Finisher', 0, NULL, 3, 15),
('ach_08', 'PUBLISHER', 'Publisher', 'Publish a finished animated episode to YouTube with metadata.', 'Distribution', 150, 'badge_beacon', 'Theme: Broadcast Gold', 0, NULL, 0, 1),
('ach_09', 'SEASON_FINISHER', 'Season Finisher', 'Complete a multi-episode animated series season.', 'Mastery', 500, 'badge_dragon', 'Title: Showrunner Supreme', 0, NULL, 0, 3);

-- 10 Skill Tree Categories
INSERT OR IGNORE INTO skill_categories (category, display_name, xp_earned, rank_level, tasks_completed, icon_key, description) VALUES
('Storytelling', 'Storytelling', 95, 2, 3, 'quill', 'Narrative arcs, premise hooks, and thematic resonance.'),
('Screenwriting', 'Screenwriting', 85, 2, 2, 'scroll', 'Dialogue pacing, scene beats, and visual script formatting.'),
('Character Design', 'Character Design', 35, 1, 1, 'palette', 'Silhouettes, color scripts, expression sheets, and turnarounds.'),
('World Building', 'World Building', 25, 1, 1, 'globe', 'Lore rules, faction aesthetics, and atmospheric continuity.'),
('Character Rigging', 'Character Rigging', 0, 1, 0, 'bones', 'Bone hierarchies, smart bones, IK/FK constraints, and weight painting.'),
('Animation', 'Animation', 0, 1, 0, 'film', 'Timing, spacing, anticipation, walk cycles, and lip-sync.'),
('Environment Design', 'Environment Design', 0, 1, 0, 'mountain', 'Background layout, multi-plane parallax, and lighting mood.'),
('Sound Design', 'Sound Design', 45, 1, 1, 'waveform', 'Voice acting direction, foley layers, ambient beds, and mixing.'),
('Editing', 'Editing', 0, 1, 0, 'scissors', 'Animatic timing, scene assembly, color grading, and compositing.'),
('Production Management', 'Production Management', 50, 1, 2, 'compass', 'Roadmap planning, scope discipline, QA checklists, and publishing.');

-- Default Startup & Reminder Settings
INSERT OR IGNORE INTO settings (key, value, category) VALUES
('start_with_windows', 'true', 'startup'),
('start_minimized', 'false', 'startup'),
('minimize_to_tray', 'true', 'startup'),
('run_reminder_scheduler_bg', 'true', 'startup'),
('launch_dashboard_on_open', 'true', 'startup'),
('quiet_hours_start', '22:30', 'reminders'),
('quiet_hours_end', '07:30', 'reminders'),
('quiet_hours_enabled', 'true', 'reminders'),
('weekly_target_xp', '200', 'gamification'),
('daily_max_tasks', '5', 'gamification'),
('projects_root_dir', 'StoryQuestProjects', 'storage'),
('auto_backup_enabled', 'true', 'storage');

-- Streak Record
INSERT OR IGNORE INTO streak_records (id, user_id, current_streak, best_streak, streak_freezes_available, weekly_rest_day, vacation_mode, last_productive_date, history_json)
VALUES ('streak_01', 'user_solo_01', 5, 12, 2, 0, 0, '2026-11-08', '["2026-11-04","2026-11-05","2026-11-06","2026-11-07","2026-11-08"]');

-- AI Provider Configurations (No hardcoded keys)
INSERT OR IGNORE INTO ai_provider_configs (id, provider_name, display_name, base_url, default_model, credential_key_name, is_active, temperature, max_tokens) VALUES
('ai_openai', 'openai', 'OpenAI API', 'https://api.openai.com/v1', 'gpt-4o', 'StoryQuest_OpenAI_API_Key', 1, 0.7, 4096),
('ai_anthropic', 'anthropic', 'Anthropic Claude API', 'https://api.anthropic.com/v1', 'claude-3-5-sonnet-latest', 'StoryQuest_Anthropic_API_Key', 0, 0.7, 4096),
('ai_compat', 'openai_compatible', 'OpenAI-Compatible Endpoint (OpenRouter / Groq)', 'https://openrouter.ai/api/v1', 'meta-llama/llama-3.3-70b-instruct', 'StoryQuest_Compat_API_Key', 0, 0.7, 4096),
('ai_local', 'local_ollama', 'Local Model Endpoint (Ollama / LM Studio)', 'http://127.0.0.1:11434/v1', 'llama3.2:latest', 'StoryQuest_Local_NoKey', 0, 0.6, 4096);

-- External Software & MCP Integrations
INSERT OR IGNORE INTO agent_connections (id, name, connection_type, executable_path, mcp_endpoint, protocol_version, is_configured, requires_approval_for_writes, status) VALUES
('mcp_blender', 'Blender MCP', 'mcp_server', 'C:\Program Files\Blender Foundation\Blender 4.2\blender.exe', 'stdio://blender-mcp-server', '2024-11-05', 1, 1, 'configured_offline'),
('mcp_moho', 'Moho Pro 14 MCP', 'mcp_server', 'C:\Program Files\Lost Marble\Moho 14\Moho.exe', 'stdio://moho-lua-bridge', '2024-11-05', 1, 1, 'configured_offline'),
('mcp_rigby', 'Rigby2D MCP', 'mcp_server', '', 'ws://127.0.0.1:9420/mcp', '2024-11-05', 0, 1, 'unconfigured'),
('agent_claude', 'Claude Code CLI', 'coding_agent', 'claude', 'cli://claude-code', '2024-11-05', 1, 1, 'configured_offline'),
('agent_opencode', 'OpenCode Agent', 'coding_agent', 'opencode', 'cli://opencode', '2024-11-05', 0, 1, 'unconfigured');
