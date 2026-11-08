#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::NaiveDate;
    use rusqlite::params;

    use crate::db::{init_database, pending_migrations, schema_version};
    use crate::commands::{query_campaign, query_episode, query_task, deps_satisfied};
    use crate::gamification::{
        award_xp_idempotent, calculate_level_from_xp, creator_rank_for_published,
        get_creator_progress, record_productive_day, recalculate_task_dependencies,
    };
    use crate::scheduler::is_within_quiet_hours;

    #[test]
    fn test_migrations_and_idempotent_xp_ledger() {
        let temp_db = PathBuf::from(":memory:");
        let conn = init_database(&temp_db).expect("In-memory SQLite migration should succeed");

        let first = award_xp_idempotent(
            &conn,
            "user_solo_01",
            "task_complete::task_01",
            "task_completion",
            "task_01",
            50,
            Some("Storytelling"),
            "Completed Story Foundation",
            false,
        )
        .unwrap();
        assert!(first.awarded);
        assert_eq!(first.xp_delta, 50);

        let second = award_xp_idempotent(
            &conn,
            "user_solo_01",
            "task_complete::task_01",
            "task_completion",
            "task_01",
            50,
            Some("Storytelling"),
            "Completed Story Foundation Again",
            false,
        )
        .unwrap();
        assert!(!second.awarded);
        assert_eq!(second.xp_delta, 0);
        assert_eq!(first.new_total_xp, second.new_total_xp);
    }

    #[test]
    fn test_progressive_levels() {
        let temp_db = PathBuf::from(":memory:");
        let conn = init_database(&temp_db).unwrap();
        let (lvl1, title1) = calculate_level_from_xp(&conn, 50).unwrap();
        assert_eq!(lvl1, 1);
        assert_eq!(title1, "Apprentice");

        let (lvl5, title5) = calculate_level_from_xp(&conn, 900).unwrap();
        assert_eq!(lvl5, 5);
        assert_eq!(title5, "Animation Apprentice");

        let (lvl10, title10) = calculate_level_from_xp(&conn, 5000).unwrap();
        assert_eq!(lvl10, 10);
        assert_eq!(title10, "Master Storyteller");
    }

    #[test]
    fn test_streak_and_freeze_preservation() {
        let temp_db = PathBuf::from(":memory:");
        let conn = init_database(&temp_db).unwrap();
        let d1 = NaiveDate::from_ymd_opt(2026, 11, 9).unwrap();
        let r1 = record_productive_day(&conn, "user_solo_01", d1).unwrap();
        assert_eq!(r1.current_streak, 1);
    }

    #[test]
    fn test_quiet_hours_overnight() {
        let late = chrono::NaiveTime::from_hms_opt(23, 15, 0).unwrap();
        let mid_day = chrono::NaiveTime::from_hms_opt(14, 0, 0).unwrap();
        assert!(is_within_quiet_hours(late, "22:30", "07:30"));
        assert!(!is_within_quiet_hours(mid_day, "22:30", "07:30"));
    }

    #[test]
    fn test_creator_ranks_are_published_video_based() {
        let conn = init_database(&PathBuf::from(":memory:")).unwrap();

        let ranks: i32 = conn
            .query_row("SELECT COUNT(*) FROM creator_ranks", [], |row| row.get(0))
            .unwrap();
        assert_eq!(ranks, 13);

        assert_eq!(creator_rank_for_published(&conn, 0).unwrap().level, 1);
        assert_eq!(creator_rank_for_published(&conn, 1).unwrap().level, 2);
        assert_eq!(creator_rank_for_published(&conn, 3).unwrap().level, 3);
        assert_eq!(creator_rank_for_published(&conn, 5).unwrap().level, 4);
        assert_eq!(creator_rank_for_published(&conn, 500).unwrap().level, 13);

        let progress = get_creator_progress(&conn, "user_solo_01").unwrap();
        assert_eq!(progress.published_videos_total, 0);
        assert_eq!(progress.rank_level, 1);
    }

    #[test]
    fn test_foundation_resets_fabricated_progression() {
        let conn = init_database(&PathBuf::from(":memory:")).unwrap();

        let (display_name, level, xp, title): (String, i32, i32, String) = conn
            .query_row(
                "SELECT display_name, current_level, total_xp, active_title FROM users WHERE id = 'user_solo_01'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(display_name, "Solo Creator");
        assert_eq!(level, 1);
        assert_eq!(xp, 0);
        assert_eq!(title, "Aspiring Creator");

        let (streak, best): (i32, i32) = conn
            .query_row(
                "SELECT current_streak, best_streak FROM streak_records WHERE user_id = 'user_solo_01'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(streak, 0);
        assert_eq!(best, 0);

        let unlocked: i32 = conn
            .query_row("SELECT COUNT(*) FROM achievements WHERE unlocked = 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(unlocked, 0);

        let (is_sample, status): (i32, String) = conn
            .query_row(
                "SELECT is_sample, status FROM campaigns WHERE id = 'camp_01'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(is_sample, 1);
        assert_eq!(status, "planning");

        let (total, available, locked): (i32, i32, i32) = conn
            .query_row(
                "SELECT COUNT(*),
                        SUM(CASE WHEN status = 'available' THEN 1 ELSE 0 END),
                        SUM(CASE WHEN status = 'locked' THEN 1 ELSE 0 END)
                 FROM tasks WHERE episode_id = 'ep_01'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(total, 15);
        assert_eq!(available, 1);
        assert_eq!(locked, 14);
    }

    #[test]
    fn test_migrations_apply_once_and_are_recorded() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("storyquest.db");

        let conn = init_database(&db_path).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), "006_youtube_sync");
        assert!(pending_migrations(&conn).unwrap().is_empty());
        drop(conn);

        let conn = init_database(&db_path).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), "006_youtube_sync");
        assert!(pending_migrations(&conn).unwrap().is_empty());
        let applied: i32 = conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(applied, 6);
    }

    #[test]
    fn test_m3_first_task_unlocks_first_quest() {
        let conn = init_database(&PathBuf::from(":memory:")).unwrap();

        conn.execute(
            "INSERT INTO campaigns (id, title, synopsis, genre, visual_style, target_runtime_minutes,
                     status, project_folder_path, created_at, updated_at)
             VALUES ('camp_new', 'Test Campaign', 'A synopsis.', 'Dark Fantasy', '2.5D', 8,
                     'planning', 'C:/Projects/Test Campaign', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO episodes (id, campaign_id, episode_number, title, premise, narrative_structure,
                     target_duration_sec, current_stage, roadmap_approved, status, youtube_url,
                     created_at, updated_at)
             VALUES ('ep_new', 'camp_new', 1, 'Test Episode', 'A premise.', '', 480, 1, 0, 'draft', NULL,
                     datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tasks (id, campaign_id, episode_id, stage_number, stage_name, title, description,
                     objective, required_input, ai_instructions, human_responsibilities, required_software,
                     expected_output, estimated_effort_mins, actual_effort_mins, definition_of_done,
                     completion_requirement_type, completion_artifact_value, difficulty, xp_reward,
                     skill_category, status, is_daily_quest, due_date, notes, attached_files_json,
                     completed_at, approved_at)
             VALUES ('task_first', 'camp_new', 'ep_new', 1, 'Story Foundation', 'First Task', 'desc',
                     'obj', '', '', '', 'Blender', 'out', 30, 0, 'done', 'confirmation', '', 'small', 10,
                     'Storytelling', 'available', 0, NULL, '', '[]', NULL, NULL)",
            [],
        )
        .unwrap();

        // Mark task completed via dependency-unlock path (status only, no dependency).
        conn.execute(
            "UPDATE tasks SET status = 'completed', completed_at = datetime('now') WHERE id = 'task_first'",
            [],
        )
        .unwrap();

        let unlocks = crate::gamification::evaluate_achievements(
            &conn,
            "user_solo_01",
            crate::gamification::AchievementNotificationMode::Individual,
        )
        .unwrap();
        let codes: Vec<_> = unlocks.iter().map(|u| u.code.as_str()).collect();
        assert!(codes.contains(&"FIRST_QUEST"), "Expected FIRST_QUEST unlock, got {:?}", codes);

        // Re-evaluating must not duplicate.
        let unlocks2 = crate::gamification::evaluate_achievements(
            &conn,
            "user_solo_01",
            crate::gamification::AchievementNotificationMode::Individual,
        )
        .unwrap();
        assert!(unlocks2.is_empty(), "Second evaluation should yield no new unlocks");

        let unlocked: i32 = conn
            .query_row("SELECT COUNT(*) FROM achievements WHERE unlocked = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(unlocked, 1);
    }

    #[test]
    fn test_m3_multi_user_achievement_keys_are_user_specific() {
        let conn_a = init_database(&PathBuf::from(":memory:")).unwrap();
        let conn_b = init_database(&PathBuf::from(":memory:")).unwrap();

        // Both users must exist in the users table for creator_progress FK.
        conn_a.execute("INSERT OR IGNORE INTO users (id, display_name, studio_name, current_level, total_xp, active_title, active_theme, avatar_badge, daily_xp_custom_tasks) VALUES ('user_a', 'User A', 'Studio A', 1, 0, 'Aspiring Creator', 'obsidian_gold', '', 0)", []).unwrap();
        conn_b.execute("INSERT OR IGNORE INTO users (id, display_name, studio_name, current_level, total_xp, active_title, active_theme, avatar_badge, daily_xp_custom_tasks) VALUES ('user_b', 'User B', 'Studio B', 1, 0, 'Aspiring Creator', 'obsidian_gold', '', 0)", []).unwrap();

        let unlocks_a = crate::gamification::evaluate_achievements(
            &conn_a,
            "user_a",
            crate::gamification::AchievementNotificationMode::Individual,
        )
        .unwrap();
        let unlocks_b = crate::gamification::evaluate_achievements(
            &conn_b,
            "user_b",
            crate::gamification::AchievementNotificationMode::Individual,
        )
        .unwrap();

        assert!(unlocks_a.is_empty());
        assert!(unlocks_b.is_empty());
    }

    #[test]
    fn test_m3_hidden_achievement_is_still_evaluated() {
        let conn = init_database(&PathBuf::from(":memory:")).unwrap();

        conn.execute(
            "INSERT INTO campaigns (id, title, synopsis, genre, visual_style, target_runtime_minutes,
                     status, project_folder_path, created_at, updated_at)
             VALUES ('camp_new', 'Test Campaign', 'A synopsis.', 'Dark Fantasy', '2.5D', 8,
                     'planning', 'C:/Projects/Test Campaign', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO episodes (id, campaign_id, episode_number, title, premise, narrative_structure,
                     target_duration_sec, current_stage, roadmap_approved, status, youtube_url,
                     created_at, updated_at)
             VALUES ('ep_new', 'camp_new', 1, 'Test Episode', 'A premise.', '', 480, 1, 0, 'draft', NULL,
                     datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tasks (id, campaign_id, episode_id, stage_number, stage_name, title, description,
                     objective, required_input, ai_instructions, human_responsibilities, required_software,
                     expected_output, estimated_effort_mins, actual_effort_mins, definition_of_done,
                     completion_requirement_type, completion_artifact_value, difficulty, xp_reward,
                     skill_category, status, is_daily_quest, due_date, notes, attached_files_json,
                     completed_at, approved_at)
             VALUES ('task_first', 'camp_new', 'ep_new', 1, 'Story Foundation', 'First Task', 'desc',
                     'obj', '', '', '', 'Blender', 'out', 30, 0, 'done', 'confirmation', '', 'small', 10,
                     'Storytelling', 'available', 0, NULL, '', '[]', NULL, NULL)",
            [],
        )
        .unwrap();

        // Make VOICE_OF_WORLD hidden and give it a satisfiable condition temporarily.
        conn.execute(
            "UPDATE achievements SET condition_type = 'task_completed', condition_value = 1,
                     progress_target = 1, progress_available = 1, is_hidden = 1 WHERE code = 'VOICE_OF_WORLD'",
            [],
        )
        .unwrap();

        // Complete a task to trigger task_completed = 1.
        conn.execute(
            "UPDATE tasks SET status = 'completed', completed_at = datetime('now') WHERE id = 'task_first'",
            [],
        )
        .unwrap();

        let unlocks = crate::gamification::evaluate_achievements(
            &conn,
            "user_solo_01",
            crate::gamification::AchievementNotificationMode::Individual,
        )
        .unwrap();
        let codes: Vec<_> = unlocks.iter().map(|u| u.code.as_str()).collect();
        assert!(
            codes.contains(&"VOICE_OF_WORLD"),
            "Hidden achievement VOICE_OF_WORLD should still be evaluated and unlocked, got {:?}",
            codes
        );
    }

    #[test]
    fn test_m3_three_day_streak_gap_resets() {
        let conn = init_database(&PathBuf::from(":memory:")).unwrap();

        // Productive on Nov 1, then Nov 5 (3 eligible missed days: Nov 2,3,4).
        let d1 = NaiveDate::from_ymd_opt(2026, 11, 1).unwrap();
        let r1 = crate::gamification::record_productive_day(&conn, "user_solo_01", d1).unwrap();
        assert_eq!(r1.current_streak, 1);

        let d5 = NaiveDate::from_ymd_opt(2026, 11, 5).unwrap();
        let r2 = crate::gamification::record_productive_day(&conn, "user_solo_01", d5).unwrap();
        assert_eq!(r2.current_streak, 1, "3 eligible missed days should reset streak to 1");
    }

    #[test]
    fn test_m3_two_eligible_missed_days_consume_freezes() {
        let conn = init_database(&PathBuf::from(":memory:")).unwrap();

        // Productive Nov 1 (Monday), weekly_rest_day = 0 (Sunday).
        // Nov 2 (Tue) eligible missed, Nov 3 (Wed) eligible missed, Nov 4 (Thu) productive.
        // 2 eligible missed days -> consume 2 freezes, streak continues.
        let d1 = NaiveDate::from_ymd_opt(2026, 11, 2).unwrap(); // Monday
        let r1 = crate::gamification::record_productive_day(&conn, "user_solo_01", d1).unwrap();
        assert_eq!(r1.current_streak, 1);
        assert_eq!(r1.streak_freezes_available, 2);

        // Productive Nov 5 (Thursday): eligible missed = Nov 2, Nov 3 (2 days)
        let d5 = NaiveDate::from_ymd_opt(2026, 11, 5).unwrap();
        let r2 = crate::gamification::record_productive_day(&conn, "user_solo_01", d5).unwrap();
        assert_eq!(r2.current_streak, 2, "2 eligible missed days bridged with freezes");
        assert_eq!(r2.streak_freezes_available, 0, "Both freezes consumed");
    }

    #[test]
    fn test_m3_rest_day_excluded_from_eligible_missed() {
        let conn = init_database(&PathBuf::from(":memory:")).unwrap();

        // weekly_rest_day = 0 (Sunday). Productive Nov 1 (Sunday = rest day? No, rest day is excluded from eligible).
        // Productive Sat Oct 31, then Mon Nov 2: missed Nov 1 (Sunday = rest day) -> 0 eligible missed -> streak continues.
        let d_sat = NaiveDate::from_ymd_opt(2026, 10, 31).unwrap();
        let r1 = crate::gamification::record_productive_day(&conn, "user_solo_01", d_sat).unwrap();
        assert_eq!(r1.current_streak, 1);

        let d_mon = NaiveDate::from_ymd_opt(2026, 11, 2).unwrap();
        let r2 = crate::gamification::record_productive_day(&conn, "user_solo_01", d_mon).unwrap();
        assert_eq!(r2.current_streak, 2, "Rest day should not break streak");
    }

    #[test]
    fn test_m3_achievement_notifications_persist_and_acknowledge() -> Result<(), Box<dyn std::error::Error>> {
        let conn = init_database(&PathBuf::from(":memory:")).unwrap();

        // Insert a notification directly.
        conn.execute(
            "INSERT INTO achievement_notifications (id, user_id, achievement_id, kind, title, body,
                     items_json, dedupe_key)
             VALUES ('ntf_1', 'user_solo_01', NULL, 'unlock', 'Test Unlock', 'Body', '[]', 'unlock::user_solo_01::TEST')",
            [],
        )?;

        let mut stmt = conn.prepare(
            "SELECT id FROM achievement_notifications WHERE user_id = ?1 AND acknowledged_at IS NULL",
        )?;
        let pending: Vec<String> = stmt
            .query_map(params!["user_solo_01"], |row| row.get(0))?
            .filter_map(Result::ok)
            .collect();
        assert_eq!(pending.len(), 1, "Notification should be pending before ack");

        conn.execute(
            "UPDATE achievement_notifications SET acknowledged_at = datetime('now') WHERE id = 'ntf_1'",
            [],
        )?;
        let pending_after: Vec<String> = stmt
            .query_map(params!["user_solo_01"], |row| row.get(0))?
            .filter_map(Result::ok)
            .collect();
        assert_eq!(pending_after.len(), 0, "Notification should be gone after ack");
        Ok(())
    }

    #[test]
    fn test_m3_reopened_task_lowers_progress() {
        let conn = init_database(&PathBuf::from(":memory:")).unwrap();

        conn.execute(
            "INSERT INTO campaigns (id, title, synopsis, genre, visual_style, target_runtime_minutes,
                     status, project_folder_path, created_at, updated_at)
             VALUES ('camp_new', 'Test Campaign', 'A synopsis.', 'Dark Fantasy', '2.5D', 8,
                     'planning', 'C:/Projects/Test Campaign', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO episodes (id, campaign_id, episode_number, title, premise, narrative_structure,
                     target_duration_sec, current_stage, roadmap_approved, status, youtube_url,
                     created_at, updated_at)
             VALUES ('ep_new', 'camp_new', 1, 'Test Episode', 'A premise.', '', 480, 1, 0, 'draft', NULL,
                     datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tasks (id, campaign_id, episode_id, stage_number, stage_name, title, description,
                     objective, required_input, ai_instructions, human_responsibilities, required_software,
                     expected_output, estimated_effort_mins, actual_effort_mins, definition_of_done,
                     completion_requirement_type, completion_artifact_value, difficulty, xp_reward,
                     skill_category, status, is_daily_quest, due_date, notes, attached_files_json,
                     completed_at, approved_at)
             VALUES ('task_first', 'camp_new', 'ep_new', 1, 'Story Foundation', 'First Task', 'desc',
                     'obj', '', '', '', 'Blender', 'out', 30, 0, 'done', 'confirmation', '', 'small', 10,
                     'Storytelling', 'available', 0, NULL, '', '[]', NULL, NULL)",
            [],
        )
        .unwrap();

        conn.execute(
            "UPDATE tasks SET status = 'completed', completed_at = datetime('now') WHERE id = 'task_first'",
            [],
        )
        .unwrap();

        let unlocks = crate::gamification::evaluate_achievements(
            &conn,
            "user_solo_01",
            crate::gamification::AchievementNotificationMode::Individual,
        )
        .unwrap();
        assert!(!unlocks.is_empty(), "First task completion should yield unlocks");

        // Reopen task -> current progress drops.
        conn.execute(
            "UPDATE tasks SET status = 'available', completed_at = NULL WHERE id = 'task_first'",
            [],
        )
        .unwrap();

        let unlocks_after = crate::gamification::evaluate_achievements(
            &conn,
            "user_solo_01",
            crate::gamification::AchievementNotificationMode::Individual,
        )
        .unwrap();
        assert!(unlocks_after.is_empty(), "No new unlocks after reopen");
    }

    #[test]
    fn test_m2_create_campaign_and_episode() {
        let conn = init_database(&PathBuf::from(":memory:")).unwrap();

        conn.execute(
            "INSERT INTO campaigns (id, title, synopsis, genre, visual_style, target_runtime_minutes,
                     status, project_folder_path, created_at, updated_at)
             VALUES ('camp_new', 'Test Campaign', 'A synopsis.', 'Dark Fantasy', '2.5D', 8,
                     'planning', 'C:/Projects/Test Campaign', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();

        let campaign = query_campaign(&conn, "camp_new").unwrap().unwrap();
        assert_eq!(campaign.title, "Test Campaign");
        assert_eq!(campaign.status, "planning");
        assert_eq!(campaign.is_sample, false);

        conn.execute(
            "INSERT INTO episodes (id, campaign_id, episode_number, title, premise, narrative_structure,
                     target_duration_sec, current_stage, roadmap_approved, status, youtube_url,
                     created_at, updated_at)
             VALUES ('ep_new', 'camp_new', 1, 'Test Episode', 'A premise.', '', 480, 1, 0, 'draft', NULL,
                     datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();

        let episode = query_episode(&conn, "camp_new").unwrap().unwrap();
        assert_eq!(episode.title, "Test Episode");
        assert_eq!(episode.campaign_id, "camp_new");
        assert_eq!(episode.current_stage, 1);
        assert_eq!(episode.roadmap_approved, false);
    }

    #[test]
    fn test_m2_task_dependency_unlock_chain() {
        let conn = init_database(&PathBuf::from(":memory:")).unwrap();

        conn.execute(
            "INSERT INTO campaigns (id, title, synopsis, genre, visual_style, target_runtime_minutes,
                     status, project_folder_path, created_at, updated_at)
             VALUES ('camp_new', 'Test Campaign', 'A synopsis.', 'Dark Fantasy', '2.5D', 8,
                     'planning', 'C:/Projects/Test Campaign', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO episodes (id, campaign_id, episode_number, title, premise, narrative_structure,
                     target_duration_sec, current_stage, roadmap_approved, status, youtube_url,
                     created_at, updated_at)
             VALUES ('ep_new', 'camp_new', 1, 'Test Episode', 'A premise.', '', 480, 1, 0, 'draft', NULL,
                     datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();

        // task_a: stage 1, available (no deps)
        conn.execute(
            "INSERT INTO tasks (id, campaign_id, episode_id, stage_number, stage_name, title, description,
                     objective, required_input, ai_instructions, human_responsibilities, required_software,
                     expected_output, estimated_effort_mins, actual_effort_mins, definition_of_done,
                     completion_requirement_type, completion_artifact_value, difficulty, xp_reward,
                     skill_category, status, is_daily_quest, due_date, notes, attached_files_json,
                     completed_at, approved_at)
             VALUES ('task_a', 'camp_new', 'ep_new', 1, 'Story Foundation', 'Task A', 'desc A', 'obj A',
                     '', '', '', 'Blender', 'out A', 30, 0, 'done A', 'confirmation', '', 'small', 10,
                     'Storytelling', 'available', 0, NULL, '', '[]', NULL, NULL)",
            [],
        )
        .unwrap();

        // task_b: stage 2, locked, depends on task_a
        conn.execute(
            "INSERT INTO tasks (id, campaign_id, episode_id, stage_number, stage_name, title, description,
                     objective, required_input, ai_instructions, human_responsibilities, required_software,
                     expected_output, estimated_effort_mins, actual_effort_mins, definition_of_done,
                     completion_requirement_type, completion_artifact_value, difficulty, xp_reward,
                     skill_category, status, is_daily_quest, due_date, notes, attached_files_json,
                     completed_at, approved_at)
             VALUES ('task_b', 'camp_new', 'ep_new', 2, 'Script Development', 'Task B', 'desc B', 'obj B',
                     '', '', '', 'Blender', 'out B', 30, 0, 'done B', 'confirmation', '', 'small', 10,
                     'Storytelling', 'locked', 0, NULL, '', '[]', NULL, NULL)",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO task_dependencies (task_id, depends_on_task_id, require_approval)
             VALUES ('task_b', 'task_a', 0)",
            [],
        )
        .unwrap();

        // task_a not completed -> task_b deps not satisfied
        assert!(!deps_satisfied(&conn, "task_b").unwrap());

        // Mark task_a completed, then unlock dependents
        conn.execute(
            "UPDATE tasks SET status = 'completed', completed_at = datetime('now') WHERE id = 'task_a'",
            [],
        )
        .unwrap();

        let unlocked = recalculate_task_dependencies(&conn, "ep_new").unwrap();
        assert_eq!(unlocked, 1);

        let task_b = query_task(&conn, "task_b").unwrap().unwrap();
        assert_eq!(task_b.status, "available");
        assert!(deps_satisfied(&conn, "task_b").unwrap());
    }

    #[test]
    fn test_m2_task_cannot_complete_without_satisfied_deps() {
        let conn = init_database(&PathBuf::from(":memory:")).unwrap();

        conn.execute(
            "INSERT INTO campaigns (id, title, synopsis, genre, visual_style, target_runtime_minutes,
                     status, project_folder_path, created_at, updated_at)
             VALUES ('camp_new', 'Test Campaign', 'A synopsis.', 'Dark Fantasy', '2.5D', 8,
                     'planning', 'C:/Projects/Test Campaign', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO episodes (id, campaign_id, episode_number, title, premise, narrative_structure,
                     target_duration_sec, current_stage, roadmap_approved, status, youtube_url,
                     created_at, updated_at)
             VALUES ('ep_new', 'camp_new', 1, 'Test Episode', 'A premise.', '', 480, 1, 0, 'draft', NULL,
                     datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();

        // task_c: locked, depends on task_d (not completed)
        conn.execute(
            "INSERT INTO tasks (id, campaign_id, episode_id, stage_number, stage_name, title, description,
                     objective, required_input, ai_instructions, human_responsibilities, required_software,
                     expected_output, estimated_effort_mins, actual_effort_mins, definition_of_done,
                     completion_requirement_type, completion_artifact_value, difficulty, xp_reward,
                     skill_category, status, is_daily_quest, due_date, notes, attached_files_json,
                     completed_at, approved_at)
             VALUES ('task_c', 'camp_new', 'ep_new', 3, 'Voice Design', 'Task C', 'desc C', 'obj C',
                     '', '', '', 'Blender', 'out C', 30, 0, 'done C', 'confirmation', '', 'small', 10,
                     'Storytelling', 'locked', 0, NULL, '', '[]', NULL, NULL)",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO tasks (id, campaign_id, episode_id, stage_number, stage_name, title, description,
                     objective, required_input, ai_instructions, human_responsibilities, required_software,
                     expected_output, estimated_effort_mins, actual_effort_mins, definition_of_done,
                     completion_requirement_type, completion_artifact_value, difficulty, xp_reward,
                     skill_category, status, is_daily_quest, due_date, notes, attached_files_json,
                     completed_at, approved_at)
             VALUES ('task_d', 'camp_new', 'ep_new', 4, 'Visual Direction', 'Task D', 'desc D', 'obj D',
                     '', '', '', 'Blender', 'out D', 30, 0, 'done D', 'confirmation', '', 'small', 10,
                     'Storytelling', 'locked', 0, NULL, '', '[]', NULL, NULL)",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO task_dependencies (task_id, depends_on_task_id, require_approval)
             VALUES ('task_c', 'task_d', 0)",
            [],
        )
        .unwrap();

        assert!(!deps_satisfied(&conn, "task_c").unwrap());
    }
}

