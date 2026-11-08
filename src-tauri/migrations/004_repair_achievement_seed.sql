-- Repair seed for databases that applied the first 003 build before the
-- achievements table's NOT NULL cosmetic_reward default was accounted for.
-- Safe on fresh databases: duplicate codes are ignored.

INSERT OR IGNORE INTO achievements
    (id, code, title, description, category, xp_bonus, badge_svg_key, cosmetic_reward, unlocked, progress_current, progress_target, condition_type, condition_value, condition_scope, progress_available)
VALUES
    ('ach_10', 'FIRST_QUEST',             'First Quest',               'Complete your first production task.', 'production', 10,  'badge_first_quest',   '', 0, 0, 1,   'task_completed',   1,    'total', 1),
    ('ach_11', 'FIRST_UPLOAD',            'First Upload',              'Publish your first video to YouTube.', 'youtube',    25,  'badge_upload',        '', 0, 0, 1,   'published_count',  1,    'total', 1),
    ('ach_12', 'FIVE_PUBLISHED',          'Five Published',            'Publish five videos.', 'youtube',               50,  'badge_five',          '', 0, 0, 5,   'published_count',  5,    'total', 1),
    ('ach_13', 'TEN_PUBLISHED',           'Ten Published',             'Publish ten videos.', 'youtube',                100, 'badge_ten',           '', 0, 0, 10,  'published_count',  10,   'total', 1),
    ('ach_14', 'HUNDRED_PUBLISHED',       'Hundred Published',         'Publish one hundred videos.', 'youtube',         500, 'badge_hundred',       '', 0, 0, 100, 'published_count',  100,  'total', 1),
    ('ach_15', 'VIEWS_100',               'First Hundred Views',       'Reach 100 total channel views.', 'youtube',       25,  'badge_views_100',     '', 0, 0, 100, 'channel_views',    100,  'total', 1),
    ('ach_16', 'VIEWS_1000',              'One Thousand Views',        'Reach 1,000 total channel views.', 'youtube',     50,  'badge_views_1000',    '', 0, 0, 1000,'channel_views',    1000, 'total', 1),
    ('ach_17', 'VIEWS_10000',             'Ten Thousand Views',        'Reach 10,000 total channel views.', 'youtube',    150, 'badge_views_10000',   '', 0, 0, 10000,'channel_views',   10000,'total', 1),
    ('ach_18', 'SUBS_100',                'First Hundred Subscribers', 'Reach 100 subscribers.', 'youtube',               50,  'badge_subs_100',      '', 0, 0, 100, 'subscribers',      100,  'total', 1),
    ('ach_19', 'SUBS_1000',               'One Thousand Subscribers',  'Reach 1,000 subscribers.', 'youtube',            150, 'badge_subs_1000',     '', 0, 0, 1000,'subscribers',      1000, 'total', 1),
    ('ach_20', 'SUBS_10000',              'Ten Thousand Subscribers',  'Reach 10,000 subscribers.', 'youtube',           500, 'badge_subs_10000',    '', 0, 0, 10000,'subscribers',     10000,'total', 1),
    ('ach_21', 'WATCH_100_HOURS',         'One Hundred Watch Hours',   'Accumulate 100 hours of watch time.', 'youtube',   100, 'badge_watch',         '', 0, 0, 6000,'watch_minutes',    6000, 'total', 1),
    ('ach_22', 'CONSISTENCY_CHAMPION',    'Consistency Champion',      'Keep a productive streak for four weeks.', 'consistency', 100, 'badge_consistency', '', 0, 0, 4, 'consistency_weeks', 4, 'total', 1),
    ('ach_23', 'FIRST_CHARACTER_APPROVED','First Character Approved',  'Get a character design approved.', 'production', 25, 'badge_character',     '', 0, 0, 1,   'stage_status',     1,    'stage:5', 1),
    ('ach_24', 'FIRST_ANIMATION_COMPLETE','First Animation Complete',  'Complete the animation stage.', 'production',    50,  'badge_animation',     '', 0, 0, 1,   'stage_status',     1,    'stage:10', 1);
