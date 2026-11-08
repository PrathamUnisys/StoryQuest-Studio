-- ============================================================
-- StoryQuest Studio â€” Milestone 1 foundation
-- Creator ranks, creator progress, achievement definitions,
-- editable settings, and removal of fabricated demo progression.
-- ============================================================

CREATE TABLE IF NOT EXISTS creator_ranks (
    level                   INTEGER PRIMARY KEY,
    rank_name               TEXT NOT NULL UNIQUE,
    min_published_videos    INTEGER NOT NULL UNIQUE,
    badge_icon              TEXT NOT NULL DEFAULT 'rank_badge',
    description             TEXT NOT NULL DEFAULT '',
    created_at              TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at              TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO creator_ranks (level, rank_name, min_published_videos, badge_icon, description) VALUES
    (1,  'Aspiring Creator',      0,   'rank_spark',     'The journey begins. Every story starts with a first idea.'),
    (2,  'Apprentice Creator',    1,   'rank_quill',     'First published video. The first step is now behind you.'),
    (3,  'Story Explorer',        3,   'rank_compass',   'Finding your voice and exploring new narrative territory.'),
    (4,  'Storyteller',           5,   'rank_scroll',    'A growing body of work with a recognizable voice.'),
    (5,  'Creative Architect',    10,  'rank_blueprint', 'Building worlds and systems that support your stories.'),
    (6,  'Animation Apprentice',  15,  'rank_pencil',    'Animation craft is becoming a dependable part of your workflow.'),
    (7,  'Character Animator',    25,  'rank_puppet',    'Characters move with intent and personality.'),
    (8,  'Animation Director',    40,  'rank_clapper',   'Directing complete productions from concept to release.'),
    (9,  'Studio Creator',        60,  'rank_studio',    'Running a repeatable, professional production pipeline.'),
    (10, 'Master Storyteller',    100, 'rank_crown',     'A hundred published stories. A body of work.'),
    (11, 'Established Creator',   150, 'rank_laurel',    'A trusted name with a durable audience.'),
    (12, 'Professional Studio',   250, 'rank_tower',     'A studio-scale operation with consistent output.'),
    (13, 'Legendary Creator',     500, 'rank_legend',    'Five hundred published works. A legacy in motion.');

CREATE TABLE IF NOT EXISTS creator_progress (
    user_id                 TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    rank_level              INTEGER NOT NULL DEFAULT 1,
    published_videos_total  INTEGER NOT NULL DEFAULT 0,
    shorts_count            INTEGER NOT NULL DEFAULT 0,
    long_form_count         INTEGER NOT NULL DEFAULT 0,
    subscriber_count        INTEGER,
    total_channel_views     INTEGER,
    total_watch_minutes     INTEGER,
    last_recalculated_at    TEXT,
    updated_at              TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO creator_progress (user_id, rank_level, published_videos_total)
SELECT id, 1, 0 FROM users;

ALTER TABLE achievements ADD COLUMN condition_type TEXT NOT NULL DEFAULT 'manual';
ALTER TABLE achievements ADD COLUMN condition_value INTEGER NOT NULL DEFAULT 0;
ALTER TABLE achievements ADD COLUMN condition_scope TEXT NOT NULL DEFAULT 'total';
ALTER TABLE achievements ADD COLUMN progress_available INTEGER NOT NULL DEFAULT 1;
ALTER TABLE achievements ADD COLUMN is_hidden INTEGER NOT NULL DEFAULT 0;

ALTER TABLE campaigns ADD COLUMN is_sample INTEGER NOT NULL DEFAULT 0;

UPDATE achievements SET
    unlocked = 0,
    unlocked_at = NULL,
    progress_current = 0,
    condition_type = 'manual',
    condition_value = 0,
    condition_scope = 'total',
    progress_available = 1,
    is_hidden = 0;

UPDATE achievements SET condition_type = 'stage_status', condition_value = 2, condition_scope = 'stage:2', progress_target = 2
    WHERE code = 'FIRST_STORY';
UPDATE achievements SET condition_type = 'episode_completed', condition_value = 1, progress_target = 1
    WHERE code = 'FIRST_EPISODE';
UPDATE achievements SET condition_type = 'stage_status', condition_value = 1, condition_scope = 'stage:10', progress_target = 1
    WHERE code = 'PUPPET_MASTER';
UPDATE achievements SET condition_type = 'task_completed', condition_value = 20, progress_target = 20
    WHERE code = 'SCENE_BUILDER';
UPDATE achievements SET condition_type = 'published_count', condition_value = 1, progress_target = 1
    WHERE code = 'PUBLISHER';
UPDATE achievements SET condition_type = 'manual', progress_available = 0, is_hidden = 0
    WHERE code = 'VOICE_OF_WORLD';
UPDATE achievements SET condition_type = 'task_completed', condition_value = 40, progress_target = 40
    WHERE code = 'ANIMATOR';
UPDATE achievements SET condition_type = 'manual', progress_available = 0
    WHERE code = 'SEASON_FINISHER';

INSERT OR IGNORE INTO achievements
    (id, code, title, description, category, xp_bonus, badge_svg_key, cosmetic_reward, unlocked, progress_current, progress_target, condition_type, condition_value, condition_scope, progress_available)
VALUES
    ('ach_10', 'FIRST_QUEST',            'First Quest',              'Complete your first production task.', 'production', 10,  'badge_first_quest',   '', 0, 0, 1,   'task_completed',   1,    'total', 1),
    ('ach_11', 'FIRST_UPLOAD',           'First Upload',             'Publish your first video to YouTube.', 'youtube',    25,  'badge_upload',        '', 0, 0, 1,   'published_count',  1,    'total', 1),
    ('ach_12', 'FIVE_PUBLISHED',         'Five Published',           'Publish five videos.', 'youtube',               50,  'badge_five',          '', 0, 0, 5,   'published_count',  5,    'total', 1),
    ('ach_13', 'TEN_PUBLISHED',          'Ten Published',            'Publish ten videos.', 'youtube',                100, 'badge_ten',           '', 0, 0, 10,  'published_count',  10,   'total', 1),
    ('ach_14', 'HUNDRED_PUBLISHED',      'Hundred Published',        'Publish one hundred videos.', 'youtube',         500, 'badge_hundred',       '', 0, 0, 100, 'published_count',  100,  'total', 1),
    ('ach_15', 'VIEWS_100',              'First Hundred Views',      'Reach 100 total channel views.', 'youtube',       25,  'badge_views_100',     '', 0, 0, 100, 'channel_views',    100,  'total', 1),
    ('ach_16', 'VIEWS_1000',             'One Thousand Views',       'Reach 1,000 total channel views.', 'youtube',     50,  'badge_views_1000',    '', 0, 0, 1000,'channel_views',    1000, 'total', 1),
    ('ach_17', 'VIEWS_10000',            'Ten Thousand Views',       'Reach 10,000 total channel views.', 'youtube',    150, 'badge_views_10000',   '', 0, 0, 10000,'channel_views',   10000,'total', 1),
    ('ach_18', 'SUBS_100',               'First Hundred Subscribers','Reach 100 subscribers.', 'youtube',               50,  'badge_subs_100',      '', 0, 0, 100, 'subscribers',      100,  'total', 1),
    ('ach_19', 'SUBS_1000',              'One Thousand Subscribers', 'Reach 1,000 subscribers.', 'youtube',            150, 'badge_subs_1000',     '', 0, 0, 1000,'subscribers',      1000, 'total', 1),
    ('ach_20', 'SUBS_10000',             'Ten Thousand Subscribers', 'Reach 10,000 subscribers.', 'youtube',           500, 'badge_subs_10000',    '', 0, 0, 10000,'subscribers',     10000,'total', 1),
    ('ach_21', 'WATCH_100_HOURS',        'One Hundred Watch Hours',  'Accumulate 100 hours of watch time.', 'youtube',   100, 'badge_watch',         '', 0, 0, 6000,'watch_minutes',    6000, 'total', 1),
    ('ach_22', 'CONSISTENCY_CHAMPION',   'Consistency Champion',     'Keep a productive streak for four weeks.', 'consistency', 100, 'badge_consistency', '', 0, 0, 4, 'consistency_weeks', 4, 'total', 1),
    ('ach_23', 'FIRST_CHARACTER_APPROVED','First Character Approved', 'Get a character design approved.', 'production', 25, 'badge_character',     '', 0, 0, 1,   'stage_status',     1,    'stage:5', 1),
    ('ach_24', 'FIRST_ANIMATION_COMPLETE','First Animation Complete',  'Complete the animation stage.', 'production',    50,  'badge_animation',     '', 0, 0, 1,   'stage_status',     1,    'stage:10', 1);

UPDATE achievements SET condition_type = 'task_completed', condition_value = 20, progress_target = 20, progress_available = 1
    WHERE code = 'SCENE_BUILDER';

INSERT OR IGNORE INTO settings (key, value, category, updated_at) VALUES
    ('start_with_windows',          'false',  'startup',  datetime('now')),
    ('start_minimized',             'false',  'startup',  datetime('now')),
    ('minimize_to_tray',            'true',   'startup',  datetime('now')),
    ('onboarding_completed',        'false',  'general',  datetime('now')),
    ('app_theme',                   'obsidian_gold', 'appearance', datetime('now')),
    ('youtube_sync_enabled',        'false',  'youtube',  datetime('now')),
    ('youtube_sync_interval_hours', '24',     'youtube',  datetime('now')),
    ('youtube_last_sync_at',        '',       'youtube',  datetime('now')),
    ('youtube_connection_status',   'not_connected', 'youtube', datetime('now')),
    ('daily_quest_min',             '3',      'quests',   datetime('now')),
    ('daily_quest_max',             '7',      'quests',   datetime('now')),
    ('daily_goal_xp',               '50',     'quests',   datetime('now')),
    ('weekly_goal_xp',              '200',    'quests',   datetime('now')),
    ('consistency_target_weeks',    '4',      'quests',   datetime('now')),
    ('focus_min_minutes',           '15',     'focus',    datetime('now')),
    ('xp_reward_small',             '10',     'xp',       datetime('now')),
    ('xp_reward_medium',            '25',     'xp',       datetime('now')),
    ('xp_reward_difficult',         '50',     'xp',       datetime('now')),
    ('xp_reward_milestone',         '100',    'xp',       datetime('now')),
    ('xp_reward_episode',           '150',    'xp',       datetime('now')),
    ('xp_reward_publish',           '250',    'xp',       datetime('now')),
    ('xp_reward_season',            '500',    'xp',       datetime('now'));

UPDATE settings SET value = 'false'
    WHERE key = 'start_with_windows';

-- Remove fabricated account progression from the original demo seed.
UPDATE users SET
    display_name = 'Solo Creator',
    studio_name = 'My Animation Studio',
    current_level = 1,
    total_xp = 0,
    active_title = 'Aspiring Creator',
    active_theme = 'obsidian_gold',
    avatar_badge = 'crest_quill',
    daily_xp_custom_tasks = 0,
    last_custom_xp_date = NULL,
    updated_at = datetime('now')
WHERE id = 'user_solo_01';

UPDATE streak_records SET
    current_streak = 0,
    best_streak = 0,
    streak_freezes_available = 2,
    weekly_rest_day = 0,
    vacation_mode = 0,
    vacation_until = NULL,
    last_productive_date = NULL,
    history_json = '[]',
    updated_at = datetime('now')
WHERE user_id = 'user_solo_01';

UPDATE skill_categories SET
    xp_earned = 0,
    rank_level = 1,
    tasks_completed = 0;

DELETE FROM xp_events WHERE user_id = 'user_solo_01';

-- Keep the original campaign as clearly labelled sample content, but reset its progress.
UPDATE campaigns SET is_sample = 1, status = 'planning', progress_percent = 0
    WHERE id = 'camp_01';
UPDATE episodes SET current_stage = 1, roadmap_approved = 0, status = 'draft', youtube_url = NULL
    WHERE campaign_id = 'camp_01';
UPDATE tasks SET
    status = CASE WHEN stage_number = 1 THEN 'available' ELSE 'locked' END,
    actual_effort_mins = 0,
    completed_at = NULL,
    approved_at = NULL,
    completion_artifact_value = ''
    WHERE campaign_id = 'camp_01';

INSERT OR IGNORE INTO creator_progress (user_id, rank_level, published_videos_total)
SELECT id, 1, 0 FROM users;

-- Starter campaign: present on fresh installs, clearly labelled as sample content.
INSERT OR IGNORE INTO campaigns
    (id, title, synopsis, genre, visual_style, world_info, target_runtime_minutes, status, project_folder_path, progress_percent, is_sample)
VALUES
    ('camp_01', 'Chronicles of Aethelgard',
     'A clockwork apprentice discovers a sentient ember capable of reigniting the frozen sun.',
     'Dark Fantasy / Clockwork',
     '2.5D Stylized Cel-Shaded with Painted Multi-Plane Backgrounds',
     'The citadel of Aethelgard runs on waning geothermal gears above the Abyssal Frost.',
     8, 'planning', 'StoryQuestProjects/Campaign_01_Chronicles_of_Aethelgard', 0.0, 1);

INSERT OR IGNORE INTO episodes
    (id, campaign_id, episode_number, title, premise, narrative_structure, target_duration_sec, current_stage, roadmap_approved, status)
VALUES
    ('ep_01', 'camp_01', 1, 'Episode 1: The Whispering Gear',
     'Lyra repairs the Great Pendulum and uncovers a forbidden voice core hidden by the First Architect.',
     'Act I: The Silent Tower -> Act II: The Spark in the Vault -> Act III: The Awakening Bell',
     480, 1, 0, 'draft');

WITH stages(stage_number, stage_name, skill_category, folder) AS (
    VALUES
        (1,  'Story Foundation',       'Storytelling',          '01_Story'),
        (2,  'Script Development',     'Screenwriting',         '01_Story'),
        (3,  'Voice Design',           'Sound Design',          '02_Audio'),
        (4,  'Visual Direction',       'World Building',        '03_Visual_Style'),
        (5,  'Character Design',       'Character Design',      '04_Characters'),
        (6,  'Character Rigging',      'Character Rigging',     '05_Rigging'),
        (7,  'Animation Library',      'Animation',             '06_Animation'),
        (8,  'Environment Creation',   'Environment Design',    '07_Environments'),
        (9,  'Storyboarding',          'Storytelling',          '08_Storyboards'),
        (10, 'Scene Animation',        'Animation',             '09_Scenes'),
        (11, 'Audio Production',       'Sound Design',          '02_Audio'),
        (12, 'Video Editing',          'Editing',               '10_Editing'),
        (13, 'Final Rendering',        'Production Management', '11_Render'),
        (14, 'Quality Assurance',      'Production Management', '00_Management'),
        (15, 'YouTube Publication',    'Production Management', '12_YouTube')
)
INSERT OR IGNORE INTO tasks
    (id, campaign_id, episode_id, stage_number, stage_name, title, description, objective,
     required_input, ai_instructions, human_responsibilities, required_software, expected_output,
     estimated_effort_mins, actual_effort_mins, difficulty, xp_reward, skill_category, status, is_daily_quest)
SELECT
    'task_ep01_s' || printf('%02d', stage_number),
    'camp_01',
    'ep_01',
    stage_number,
    stage_name,
    'Stage ' || printf('%02d', stage_number) || ': ' || stage_name,
    'Complete ' || stage_name || ' deliverables for Episode 1: The Whispering Gear.',
    'Produce verified ' || stage_name || ' output in ' || folder || '/',
    'Campaign bible & prior stage deliverable',
    'Generate structured prompts and continuity checks for ' || stage_name || '.',
    'Direct creative execution and approve final ' || stage_name || ' asset.',
    'Moho Pro 14 / Blender 4.2 / StoryQuest Workshop',
    folder || '/ep01_stage_' || printf('%02d', stage_number) || '_asset',
    CASE WHEN stage_number <= 5 THEN 30 ELSE 45 END,
    0,
    CASE WHEN stage_number IN (10, 13, 15) THEN 'large' ELSE 'medium' END,
    CASE WHEN stage_number IN (10, 13, 15) THEN 50 ELSE 25 END,
    skill_category,
    CASE WHEN stage_number = 1 THEN 'available' ELSE 'locked' END,
    CASE WHEN stage_number IN (4, 5, 6) THEN 1 ELSE 0 END
FROM stages;

WITH stages(stage_number) AS (
    VALUES (1),(2),(3),(4),(5),(6),(7),(8),(9),(10),(11),(12),(13),(14),(15)
)
INSERT OR IGNORE INTO task_dependencies (task_id, depends_on_task_id, require_approval)
SELECT
    'task_ep01_s' || printf('%02d', stage_number),
    'task_ep01_s' || printf('%02d', stage_number - 1),
    1
FROM stages
WHERE stage_number > 1;

INSERT OR IGNORE INTO reminders
    (id, title, reminder_type, task_id, schedule_time, days_of_week_json, enabled, windows_task_name)
VALUES
    ('rem_01', 'Evening Animation Quest Session', 'daily', 'task_ep01_s04', '19:00', '[1,2,3,4,5]', 1, 'Reminder_rem_01');

