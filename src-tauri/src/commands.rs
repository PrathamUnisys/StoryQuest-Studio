use std::path::PathBuf;
use std::sync::Mutex;

use chrono::{Duration, Datelike, Local, NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::ai_planner::build_progressive_production_roadmap;
use crate::credentials::{delete_api_key, retrieve_api_key, store_api_key};
use crate::db::{create_database_backup, schema_version};
use crate::gamification::{
    award_xp_idempotent, creator_rank_for_published, evaluate_achievements,
    get_creator_progress, list_creator_ranks, record_achievement_notifications,
    record_productive_day, recalculate_creator_progress, recalculate_task_dependencies,
    AchievementNotificationMode,
};
use crate::models::{
    Achievement, ActivityDay, AchievementNotification, AppInfo, BootstrapState, Campaign,
    CreatorProgress, CreatorRank, Episode, GeneratedRoadmap, ReminderConfig,
    SetStreakPrefsInput, SettingItem, StoryPlannerInput, StreakRecord, Task, TaskCounts,
    ThresholdUpdate, UserProfile, XpAwardResult, YoutubeStatus, AcknowledgeNotificationsInput,
};
use crate::project_fs::{initialize_campaign_directories, open_folder_in_explorer};
use crate::scheduler::dispatch_windows_toast;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub app_dir: PathBuf,
    pub db_path: PathBuf,
}

const DEFAULT_USER_ID: &str = "user_solo_01";

fn allowed_setting_keys() -> &'static [&'static str] {
    &[
        "start_with_windows",
        "start_minimized",
        "minimize_to_tray",
        "onboarding_completed",
        "app_theme",
        "quiet_hours_start",
        "quiet_hours_end",
        "quiet_hours_enabled",
        "weekly_target_xp",
        "daily_max_tasks",
        "daily_quest_min",
        "daily_quest_max",
        "daily_goal_xp",
        "weekly_goal_xp",
        "consistency_target_weeks",
        "focus_min_minutes",
        "projects_root_dir",
        "auto_backup_enabled",
        "run_reminder_scheduler_bg",
        "launch_dashboard_on_open",
        "youtube_sync_enabled",
        "youtube_sync_interval_hours",
        "youtube_last_sync_at",
        "youtube_connection_status",
        "youtube_channel_title",
        "youtube_channel_id",
        "xp_reward_small",
        "xp_reward_medium",
        "xp_reward_difficult",
        "xp_reward_milestone",
        "xp_reward_episode",
        "xp_reward_publish",
        "xp_reward_season",
    ]
}

fn is_allowed_setting(key: &str) -> bool {
    allowed_setting_keys().contains(&key)
}

fn query_profile(conn: &Connection, user_id: &str) -> Result<UserProfile, String> {
    conn.query_row(
        "SELECT id, display_name, studio_name, current_level, total_xp, active_title,
                active_theme, avatar_badge, daily_xp_custom_tasks, last_custom_xp_date
         FROM users WHERE id = ?1",
        params![user_id],
        |row| {
            Ok(UserProfile {
                id: row.get(0)?,
                display_name: row.get(1)?,
                studio_name: row.get(2)?,
                current_level: row.get(3)?,
                total_xp: row.get(4)?,
                active_title: row.get(5)?,
                active_theme: row.get(6)?,
                avatar_badge: row.get(7)?,
                daily_xp_custom_tasks: row.get(8)?,
                last_custom_xp_date: row.get(9)?,
            })
        },
    )
    .map_err(|e| format!("Failed to read creator profile: {e}"))
}

pub(crate) fn query_campaign(conn: &Connection, campaign_id: &str) -> Result<Option<Campaign>, String> {    conn.query_row(
        "SELECT id, title, synopsis, genre, visual_style, world_info, target_runtime_minutes,
                status, project_folder_path, progress_percent, is_sample, created_at, updated_at
         FROM campaigns WHERE id = ?1",
        params![campaign_id],
        |row| {
            Ok(Campaign {
                id: row.get(0)?,
                title: row.get(1)?,
                synopsis: row.get(2)?,
                genre: row.get(3)?,
                visual_style: row.get(4)?,
                world_info: row.get(5)?,
                target_runtime_minutes: row.get(6)?,
                status: row.get(7)?,
                project_folder_path: row.get(8)?,
                progress_percent: row.get(9)?,
                is_sample: row.get::<_, i32>(10)? != 0,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        },
    )
    .optional()
    .map_err(|e| format!("Failed to read campaign: {e}"))
}

fn query_active_campaign(conn: &Connection) -> Result<Option<Campaign>, String> {
    let id: Option<String> = conn
        .query_row(
            "SELECT id FROM campaigns
             ORDER BY CASE status WHEN 'active' THEN 0 WHEN 'planning' THEN 1 WHEN 'paused' THEN 2 ELSE 3 END,
                      created_at ASC
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("Failed to find active campaign: {e}"))?;

    match id {
        Some(id) => query_campaign(conn, &id),
        None => Ok(None),
    }
}

pub(crate) fn query_episode(conn: &Connection, campaign_id: &str) -> Result<Option<Episode>, String> {
    conn.query_row(
        "SELECT id, campaign_id, episode_number, title, premise, narrative_structure,
                target_duration_sec, current_stage, roadmap_approved, status, youtube_url
         FROM episodes WHERE campaign_id = ?1 ORDER BY episode_number ASC LIMIT 1",
        params![campaign_id],
        |row| {
            Ok(Episode {
                id: row.get(0)?,
                campaign_id: row.get(1)?,
                episode_number: row.get(2)?,
                title: row.get(3)?,
                premise: row.get(4)?,
                narrative_structure: row.get(5)?,
                target_duration_sec: row.get(6)?,
                current_stage: row.get(7)?,
                roadmap_approved: row.get::<_, i32>(8)? != 0,
                status: row.get(9)?,
                youtube_url: row.get(10)?,
            })
        },
    )
    .optional()
    .map_err(|e| format!("Failed to read episode: {e}"))
}

fn query_streak(conn: &Connection, user_id: &str) -> Result<StreakRecord, String> {
    conn.query_row(
        "SELECT id, user_id, current_streak, best_streak, streak_freezes_available,
                weekly_rest_day, vacation_mode, vacation_until, last_productive_date, history_json
         FROM streak_records WHERE user_id = ?1 LIMIT 1",
        params![user_id],
        |row| {
            Ok(StreakRecord {
                id: row.get(0)?,
                user_id: row.get(1)?,
                current_streak: row.get(2)?,
                best_streak: row.get(3)?,
                streak_freezes_available: row.get(4)?,
                weekly_rest_day: row.get(5)?,
                vacation_mode: row.get::<_, i32>(6)? != 0,
                vacation_until: row.get(7)?,
                last_productive_date: row.get(8)?,
                history_json: row.get(9)?,
            })
        },
    )
    .map_err(|e| format!("Failed to read streak record: {e}"))
}

fn query_settings(conn: &Connection) -> Result<Vec<SettingItem>, String> {
    let mut stmt = conn
        .prepare("SELECT key, value, category FROM settings ORDER BY category, key")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(SettingItem {
                key: row.get(0)?,
                value: row.get(1)?,
                category: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

fn query_achievements(conn: &Connection) -> Result<Vec<Achievement>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, code, title, description, category, xp_bonus, badge_svg_key,
                    cosmetic_reward, unlocked, unlocked_at, progress_current, progress_target,
                    condition_type, condition_value, condition_scope, progress_available,
                    is_hidden
             FROM achievements
             ORDER BY unlocked ASC, category ASC, title ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Achievement {
                id: row.get(0)?,
                code: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                category: row.get(4)?,
                xp_bonus: row.get(5)?,
                badge_svg_key: row.get(6)?,
                cosmetic_reward: row.get(7)?,
                unlocked: row.get::<_, i32>(8)? != 0,
                unlocked_at: row.get(9)?,
                progress_current: row.get(10)?,
                progress_target: row.get(11)?,
                condition_type: row.get(12)?,
                condition_value: row.get(13)?,
                condition_scope: row.get(14)?,
                progress_available: row.get::<_, i32>(15)? != 0,
                is_hidden: row.get::<_, i32>(16)? != 0,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

fn query_task_counts(conn: &Connection, episode_id: &str) -> Result<TaskCounts, String> {
    let mut stmt = conn
        .prepare("SELECT status, COUNT(*) FROM tasks WHERE episode_id = ?1 GROUP BY status")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![episode_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut counts = TaskCounts::default();
    for row in rows {
        let (status, count) = row.map_err(|e| e.to_string())?;
        counts.total += count;
        match status.as_str() {
            "locked" => counts.locked += count,
            "blocked" => counts.blocked += count,
            "available" => counts.available += count,
            "in_progress" => counts.in_progress += count,
            "review" => counts.review += count,
            "approved" => counts.approved += count,
            "completed" => counts.completed += count,
            _ => {}
        }
    }
    Ok(counts)
}

fn query_episode_progress(conn: &Connection, episode_id: &str) -> Result<f64, String> {
    let (done, total): (i32, i32) = conn
        .query_row(
            "SELECT
                COALESCE(SUM(CASE WHEN status IN ('approved', 'completed') THEN 1 ELSE 0 END), 0),
                COUNT(*)
             FROM tasks WHERE episode_id = ?1",
            params![episode_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    if total == 0 {
        Ok(0.0)
    } else {
        Ok((done as f64 / total as f64) * 100.0)
    }
}

fn load_dependencies(conn: &Connection, task_id: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT depends_on_task_id FROM task_dependencies WHERE task_id = ?1")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![task_id], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

fn row_to_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get(0)?,
        campaign_id: row.get(1)?,
        episode_id: row.get(2)?,
        stage_number: row.get(3)?,
        stage_name: row.get(4)?,
        title: row.get(5)?,
        description: row.get(6)?,
        objective: row.get(7)?,
        required_input: row.get(8)?,
        ai_instructions: row.get(9)?,
        human_responsibilities: row.get(10)?,
        required_software: row.get(11)?,
        expected_output: row.get(12)?,
        estimated_effort_mins: row.get(13)?,
        actual_effort_mins: row.get(14)?,
        definition_of_done: row.get(15)?,
        completion_requirement_type: row.get(16)?,
        completion_artifact_value: row.get(17)?,
        difficulty: row.get(18)?,
        xp_reward: row.get(19)?,
        skill_category: row.get(20)?,
        status: row.get(21)?,
        is_daily_quest: row.get::<_, i32>(22)? != 0,
        due_date: row.get(23)?,
        notes: row.get(24)?,
        attached_files_json: row.get(25)?,
        dependencies: Vec::new(),
        completed_at: row.get(26)?,
        approved_at: row.get(27)?,
    })
}

const TASK_COLUMNS: &str = "id, campaign_id, episode_id, stage_number, stage_name, title,
    description, objective, required_input, ai_instructions, human_responsibilities,
    required_software, expected_output, estimated_effort_mins, actual_effort_mins,
    definition_of_done, completion_requirement_type, completion_artifact_value, difficulty,
    xp_reward, skill_category, status, is_daily_quest, due_date, notes, attached_files_json,
    completed_at, approved_at";

fn query_deadlines(conn: &Connection, episode_id: &str) -> Result<Vec<Task>, String> {
    let sql = format!(
        "SELECT {TASK_COLUMNS} FROM tasks
         WHERE episode_id = ?1 AND due_date IS NOT NULL AND status NOT IN ('approved', 'completed')
         ORDER BY due_date ASC LIMIT 5"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![episode_id], row_to_task)
        .map_err(|e| e.to_string())?;
    let mut tasks = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    for task in &mut tasks {
        task.dependencies = load_dependencies(conn, &task.id)?;
    }
    Ok(tasks)
}

fn query_weekly_activity(conn: &Connection, user_id: &str) -> Result<Vec<ActivityDay>, String> {
    let today = Local::now().date_naive();
    let streak: Option<(i32, i32, i32, Option<String>, bool)> = conn
        .query_row(
            "SELECT weekly_rest_day, streak_freezes_available, current_streak,
                    vacation_until, vacation_mode
             FROM streak_records WHERE user_id = ?1 LIMIT 1",
            params![user_id],
            |row| {
                Ok((
                    row.get::<_, i32>(0)?,
                    row.get::<_, i32>(1)?,
                    row.get::<_, i32>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, i32>(4)? != 0,
                ))
            },
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let (weekly_rest_day, streak_freezes_available, current_streak, vacation_until, vacation_mode) =
        streak.unwrap_or((0, 2, 0, None, false));

    let mut days = Vec::with_capacity(7);

    for offset in (0..7).rev() {
        let date = today - Duration::days(offset);
        let date_str = date.to_string();
        let xp: i32 = conn
            .query_row(
                "SELECT COALESCE(SUM(xp_amount), 0) FROM xp_events
                 WHERE user_id = ?1 AND date(created_at, 'localtime') = ?2",
                params![user_id, date_str],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        let tasks_completed: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM tasks
                 WHERE status IN ('approved', 'completed')
                   AND date(completed_at, 'localtime') = ?1",
                params![date_str],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        let focus_minutes: i32 = conn
            .query_row(
                "SELECT COALESCE(SUM(actual_effort_mins), 0) FROM tasks
                 WHERE status IN ('approved', 'completed')
                   AND date(completed_at, 'localtime') = ?1",
                params![date_str],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        let has_activity = xp > 0 || tasks_completed > 0;
        let weekday = date.weekday().num_days_from_sunday() as i32;
        let is_rest = weekday == weekly_rest_day;
        let is_vacation = vacation_mode
            && vacation_until.as_deref().map_or(false, |until| {
                NaiveDate::parse_from_str(until, "%Y-%m-%d")
                    .map_or(false, |d| date <= d)
            });

        let status = if has_activity {
            "productive".to_string()
        } else if is_vacation {
            "vacation".to_string()
        } else if is_rest {
            "rest".to_string()
        } else {
            "missed".to_string()
        };

        days.push(ActivityDay {
            date: date_str,
            xp,
            tasks_completed,
            focus_minutes,
            active: has_activity,
            status,
        });
    }

    Ok(days)
}

fn query_youtube_status(conn: &Connection) -> Result<YoutubeStatus, String> {
    let status: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'youtube_connection_status'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let last_sync: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'youtube_last_sync_at'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let channel_title: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'youtube_channel_title'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let channel_id: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'youtube_channel_id'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let connected = status.as_deref() == Some("connected");
    Ok(YoutubeStatus {
        connected,
        channel_title: channel_title.filter(|v| !v.trim().is_empty()),
        channel_id: channel_id.filter(|v| !v.trim().is_empty()),
        last_sync_at: last_sync.filter(|v| !v.trim().is_empty()),
        note: if connected {
            "Connected with the official YouTube APIs.".to_string()
        } else {
            "Not connected. Add your Google OAuth Client ID/Secret, then connect; no estimates are shown."
                .to_string()
        },
        has_client_credentials: crate::youtube::has_client_credentials(),
    })
}

fn build_bootstrap(conn: &Connection, state: &AppState) -> Result<BootstrapState, String> {
    let profile = query_profile(conn, DEFAULT_USER_ID)?;
    let creator_progress = get_creator_progress(conn, DEFAULT_USER_ID)?;
    let creator_ranks = list_creator_ranks(conn)?;
    let creator_rank = creator_ranks
        .iter()
        .find(|rank| rank.level == creator_progress.rank_level)
        .cloned()
        .unwrap_or_else(|| {
            creator_rank_for_published(conn, creator_progress.published_videos_total)
                .unwrap_or(CreatorRank {
                    level: 1,
                    rank_name: "Aspiring Creator".to_string(),
                    min_published_videos: 0,
                    badge_icon: "rank_spark".to_string(),
                    description: String::new(),
                })
        });

    let settings = query_settings(conn)?;
    let onboarding_completed = settings
        .iter()
        .find(|item| item.key == "onboarding_completed")
        .map(|item| item.value == "true")
        .unwrap_or(false);

    let active_campaign = query_active_campaign(conn)?;
    let active_episode = match active_campaign.as_ref() {
        Some(campaign) => query_episode(conn, &campaign.id)?,
        None => None,
    };

    let (task_counts, episode_progress_percent, deadlines) = match active_episode.as_ref() {
        Some(episode) => (
            query_task_counts(conn, &episode.id)?,
            query_episode_progress(conn, &episode.id)?,
            query_deadlines(conn, &episode.id)?,
        ),
        None => (TaskCounts::default(), 0.0, Vec::new()),
    };

    let streak = query_streak(conn, DEFAULT_USER_ID)?;

    // Backfill: evaluate any achievements that prior play may have satisfied.
    let backfill_unlocks =
        evaluate_achievements(&conn, DEFAULT_USER_ID, AchievementNotificationMode::Summary)?;
    let _ = record_achievement_notifications(
        &conn,
        DEFAULT_USER_ID,
        &backfill_unlocks,
        AchievementNotificationMode::Summary,
    )?;

    let achievements = query_achievements(conn)?;
    let weekly_activity = query_weekly_activity(conn, DEFAULT_USER_ID)?;
    let youtube = query_youtube_status(conn)?;

    let app_info = AppInfo {
        name: "StoryQuest Studio".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        identifier: "com.storyquest.studio".to_string(),
        data_dir: state.app_dir.to_string_lossy().to_string(),
        db_path: state.db_path.to_string_lossy().to_string(),
        schema_version: schema_version(conn)?,
    };

    Ok(BootstrapState {
        profile,
        creator_progress,
        creator_rank,
        creator_ranks,
        settings,
        active_campaign,
        active_episode,
        task_counts,
        episode_progress_percent,
        streak,
        achievements,
        deadlines,
        weekly_activity,
        youtube,
        app_info,
        today: Local::now().format("%Y-%m-%d").to_string(),
        onboarding_completed,
    })
}

#[tauri::command]
pub fn cmd_get_bootstrap(state: State<'_, AppState>) -> Result<BootstrapState, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    build_bootstrap(&conn, &state)
}

#[tauri::command]
pub fn cmd_update_profile(
    state: State<'_, AppState>,
    display_name: String,
    studio_name: String,
    avatar_badge: Option<String>,
) -> Result<UserProfile, String> {
    let display_name = display_name.trim();
    let studio_name = studio_name.trim();
    if display_name.is_empty() {
        return Err("Creator name cannot be empty.".to_string());
    }
    if studio_name.is_empty() {
        return Err("Studio name cannot be empty.".to_string());
    }

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE users SET display_name = ?1, studio_name = ?2, avatar_badge = COALESCE(NULLIF(?3, ''), avatar_badge),
         updated_at = datetime('now') WHERE id = ?4",
        params![display_name, studio_name, avatar_badge.unwrap_or_default(), DEFAULT_USER_ID],
    )
    .map_err(|e| format!("Failed to save profile: {e}"))?;

    query_profile(&conn, DEFAULT_USER_ID)
}

#[tauri::command]
pub fn cmd_complete_onboarding(state: State<'_, AppState>) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value, category, updated_at)
         VALUES ('onboarding_completed', 'true', 'general', datetime('now'))
         ON CONFLICT(key) DO UPDATE SET value = 'true', updated_at = datetime('now')",
        [],
    )
    .map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn cmd_get_settings(state: State<'_, AppState>) -> Result<Vec<SettingItem>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    query_settings(&conn)
}

#[tauri::command]
pub fn cmd_set_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<SettingItem, String> {
    if !is_allowed_setting(&key) {
        return Err(format!("Unknown or protected setting key: {key}"));
    }

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let category: String = conn
        .query_row(
            "SELECT category FROM settings WHERE key = ?1",
            params![&key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "general".to_string());

    conn.execute(
        "INSERT INTO settings (key, value, category, updated_at)
         VALUES (?1, ?2, ?3, datetime('now'))
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')",
        params![&key, &value, &category],
    )
    .map_err(|e| format!("Failed to save setting {key}: {e}"))?;

    Ok(SettingItem {
        key,
        value,
        category,
    })
}

#[tauri::command]
pub fn cmd_get_creator_ranks(state: State<'_, AppState>) -> Result<Vec<CreatorRank>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    list_creator_ranks(&conn)
}

#[tauri::command]
pub fn cmd_update_creator_thresholds(
    state: State<'_, AppState>,
    thresholds: Vec<ThresholdUpdate>,
) -> Result<CreatorProgress, String> {
    if thresholds.is_empty() {
        return Err("At least one creator rank threshold is required.".to_string());
    }

    let mut sorted = thresholds;
    sorted.sort_by_key(|item| item.level);

    for (index, item) in sorted.iter().enumerate() {
        if index == 0 && item.level != 1 {
            return Err("Rank level 1 must be included.".to_string());
        }
        if item.level == 1 && item.min_published_videos != 0 {
            return Err("Rank 1 must start at 0 published videos.".to_string());
        }
        if item.level < 1 || item.level > 13 {
            return Err("Creator ranks must be between level 1 and 13.".to_string());
        }
        if index > 0 && item.min_published_videos <= sorted[index - 1].min_published_videos {
            return Err("Published-video thresholds must increase with each rank.".to_string());
        }
    }

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    for item in &sorted {
        conn.execute(
            "UPDATE creator_ranks SET min_published_videos = ?1, updated_at = datetime('now') WHERE level = ?2",
            params![item.min_published_videos, item.level],
        )
        .map_err(|e| format!("Failed to update rank {}: {e}", item.level))?;
    }

    recalculate_creator_progress(&conn, DEFAULT_USER_ID)
}

#[tauri::command]
pub fn cmd_recalculate_creator_rank(
    state: State<'_, AppState>,
) -> Result<CreatorProgress, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    recalculate_creator_progress(&conn, DEFAULT_USER_ID)
}

#[tauri::command]
pub fn cmd_get_app_info(
    state: State<'_, AppState>,
) -> Result<AppInfo, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    Ok(AppInfo {
        name: "StoryQuest Studio".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        identifier: "com.storyquest.studio".to_string(),
        data_dir: state.app_dir.to_string_lossy().to_string(),
        db_path: state.db_path.to_string_lossy().to_string(),
        schema_version: schema_version(&conn)?,
    })
}

#[tauri::command]
pub fn cmd_open_data_folder(state: State<'_, AppState>) -> Result<(), String> {
    open_folder_in_explorer(&state.app_dir)
}

#[tauri::command]
pub fn cmd_list_campaigns(state: State<'_, AppState>) -> Result<Vec<Campaign>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, title, synopsis, genre, visual_style, world_info, target_runtime_minutes,
                    status, project_folder_path, progress_percent, is_sample, created_at, updated_at
             FROM campaigns ORDER BY created_at ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Campaign {
                id: row.get(0)?,
                title: row.get(1)?,
                synopsis: row.get(2)?,
                genre: row.get(3)?,
                visual_style: row.get(4)?,
                world_info: row.get(5)?,
                target_runtime_minutes: row.get(6)?,
                status: row.get(7)?,
                project_folder_path: row.get(8)?,
                progress_percent: row.get(9)?,
                is_sample: row.get::<_, i32>(10)? != 0,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_list_episodes(
    state: State<'_, AppState>,
    campaign_id: String,
) -> Result<Vec<Episode>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, campaign_id, episode_number, title, premise, narrative_structure,
                    target_duration_sec, current_stage, roadmap_approved, status, youtube_url
             FROM episodes WHERE campaign_id = ?1 ORDER BY episode_number ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![campaign_id], |row| {
            Ok(Episode {
                id: row.get(0)?,
                campaign_id: row.get(1)?,
                episode_number: row.get(2)?,
                title: row.get(3)?,
                premise: row.get(4)?,
                narrative_structure: row.get(5)?,
                target_duration_sec: row.get(6)?,
                current_stage: row.get(7)?,
                roadmap_approved: row.get::<_, i32>(8)? != 0,
                status: row.get(9)?,
                youtube_url: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_list_episode_tasks(
    state: State<'_, AppState>,
    episode_id: String,
) -> Result<Vec<Task>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let sql = format!(
        "SELECT {TASK_COLUMNS} FROM tasks WHERE episode_id = ?1 ORDER BY stage_number ASC"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![episode_id], row_to_task)
        .map_err(|e| e.to_string())?;
    let mut tasks = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    for task in &mut tasks {
        task.dependencies = load_dependencies(&conn, &task.id)?;
    }
    Ok(tasks)
}

#[tauri::command]
pub fn cmd_list_reminders(state: State<'_, AppState>) -> Result<Vec<ReminderConfig>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, title, reminder_type, task_id, schedule_time, days_of_week_json,
                    specific_date, enabled, snoozed_until, windows_task_name, last_triggered_at
             FROM reminders ORDER BY schedule_time ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ReminderConfig {
                id: row.get(0)?,
                title: row.get(1)?,
                reminder_type: row.get(2)?,
                task_id: row.get(3)?,
                schedule_time: row.get(4)?,
                days_of_week_json: row.get(5)?,
                specific_date: row.get(6)?,
                enabled: row.get::<_, i32>(7)? != 0,
                snoozed_until: row.get(8)?,
                windows_task_name: row.get(9)?,
                last_triggered_at: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

fn validate_reminder_input(
    title: &str,
    reminder_type: &str,
    schedule_time: &str,
    days_of_week_json: &str,
    specific_date: Option<&str>,
) -> Result<(), String> {
    if title.trim().is_empty() {
        return Err("Reminder title is required".to_string());
    }
    if !["daily", "weekly", "specific_date", "task_deadline", "custom", "weekly_review"].contains(&reminder_type) {
        return Err(format!("Unknown reminder type: {reminder_type}"));
    }
    chrono::NaiveTime::parse_from_str(schedule_time.trim(), "%H:%M")
        .map_err(|_| format!("scheduleTime must be HH:MM, got {schedule_time}"))?;
    let days: Vec<i32> = serde_json::from_str(days_of_week_json)
        .map_err(|_| "daysOfWeek must be a JSON array of 0-6".to_string())?;
    for d in &days {
        if !(0..=6).contains(d) {
            return Err(format!("daysOfWeek values must be 0-6, got {d}"));
        }
    }
    if reminder_type == "specific_date" {
        let date = specific_date.unwrap_or("").trim();
        if date.is_empty() {
            return Err("specificDate is required for specific_date reminders".to_string());
        }
        chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|_| format!("specificDate must be YYYY-MM-DD, got {date}"))?;
    }
    Ok(())
}

fn reminder_from_row(row: &rusqlite::Row) -> Result<ReminderConfig, rusqlite::Error> {
    Ok(ReminderConfig {
        id: row.get(0)?,
        title: row.get(1)?,
        reminder_type: row.get(2)?,
        task_id: row.get(3)?,
        schedule_time: row.get(4)?,
        days_of_week_json: row.get(5)?,
        specific_date: row.get(6)?,
        enabled: row.get::<_, i32>(7)? != 0,
        snoozed_until: row.get(8)?,
        windows_task_name: row.get(9)?,
        last_triggered_at: row.get(10)?,
    })
}

fn load_reminder(conn: &Connection, reminder_id: &str) -> Result<ReminderConfig, String> {
    conn.query_row(
        "SELECT id, title, reminder_type, task_id, schedule_time, days_of_week_json,
                specific_date, enabled, snoozed_until, windows_task_name, last_triggered_at
         FROM reminders WHERE id = ?1",
        params![reminder_id],
        reminder_from_row,
    )
    .optional()
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Reminder {reminder_id} not found"))
}

fn sync_reminder_task(conn: &Connection, reminder: &ReminderConfig) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    crate::windows_task_scheduler::sync_windows_scheduled_task(&exe.to_string_lossy(), reminder)?;
    let _ = (conn, reminder);
    Ok(())
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReminderInput {
    pub title: String,
    pub reminder_type: String,
    pub task_id: Option<String>,
    pub schedule_time: String,
    pub days_of_week: Vec<i32>,
    pub specific_date: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateReminderInput {
    pub id: String,
    pub title: String,
    pub reminder_type: String,
    pub task_id: Option<String>,
    pub schedule_time: String,
    pub days_of_week: Vec<i32>,
    pub specific_date: Option<String>,
    pub enabled: bool,
}

#[tauri::command]
pub fn cmd_create_reminder(
    state: State<'_, AppState>,
    input: CreateReminderInput,
) -> Result<ReminderConfig, String> {
    let days_json = serde_json::to_string(&input.days_of_week).map_err(|e| e.to_string())?;
    validate_reminder_input(
        &input.title,
        &input.reminder_type,
        &input.schedule_time,
        &days_json,
        input.specific_date.as_deref(),
    )?;
    if let Some(task_id) = &input.task_id {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let exists: Option<String> = conn
            .query_row("SELECT id FROM tasks WHERE id = ?1", params![task_id], |row| row.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        if exists.is_none() {
            return Err(format!("Task {task_id} not found"));
        }
    }
    let id = format!("rem_{}", uuid::Uuid::new_v4().simple());
    let windows_task_name = format!("Reminder_{}", &id[4..12.min(id.len())]);
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO reminders (id, title, reminder_type, task_id, schedule_time, days_of_week_json,
                specific_date, enabled, windows_task_name)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8)",
        params![
            id,
            input.title.trim(),
            input.reminder_type,
            input.task_id,
            input.schedule_time.trim(),
            days_json,
            input.specific_date,
            windows_task_name,
        ],
    )
    .map_err(|e| e.to_string())?;
    let reminder = load_reminder(&conn, &id)?;
    sync_reminder_task(&conn, &reminder)?;
    Ok(reminder)
}

#[tauri::command]
pub fn cmd_update_reminder(
    state: State<'_, AppState>,
    input: UpdateReminderInput,
) -> Result<ReminderConfig, String> {
    let days_json = serde_json::to_string(&input.days_of_week).map_err(|e| e.to_string())?;
    validate_reminder_input(
        &input.title,
        &input.reminder_type,
        &input.schedule_time,
        &days_json,
        input.specific_date.as_deref(),
    )?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let existing = load_reminder(&conn, &input.id)?;
    conn.execute(
        "UPDATE reminders SET title = ?1, reminder_type = ?2, task_id = ?3, schedule_time = ?4,
                days_of_week_json = ?5, specific_date = ?6, enabled = ?7
         WHERE id = ?8",
        params![
            input.title.trim(),
            input.reminder_type,
            input.task_id,
            input.schedule_time.trim(),
            days_json,
            input.specific_date,
            if input.enabled { 1 } else { 0 },
            input.id,
        ],
    )
    .map_err(|e| e.to_string())?;
    let updated = load_reminder(&conn, &existing.id)?;
    sync_reminder_task(&conn, &updated)?;
    Ok(updated)
}

#[tauri::command]
pub fn cmd_set_reminder_enabled(
    state: State<'_, AppState>,
    reminder_id: String,
    enabled: bool,
) -> Result<ReminderConfig, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE reminders SET enabled = ?1 WHERE id = ?2",
        params![if enabled { 1 } else { 0 }, reminder_id],
    )
    .map_err(|e| e.to_string())?;
    let reminder = load_reminder(&conn, &reminder_id)?;
    sync_reminder_task(&conn, &reminder)?;
    Ok(reminder)
}

#[tauri::command]
pub fn cmd_delete_reminder(state: State<'_, AppState>, reminder_id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let reminder = load_reminder(&conn, &reminder_id)?;
    conn.execute("DELETE FROM reminders WHERE id = ?1", params![reminder_id])
        .map_err(|e| e.to_string())?;
    crate::windows_task_scheduler::remove_windows_scheduled_task(&reminder.windows_task_name)?;
    Ok(())
}

#[tauri::command]
pub fn cmd_snooze_reminder(
    state: State<'_, AppState>,
    reminder_id: String,
    minutes: i32,
) -> Result<ReminderConfig, String> {
    if !(5..=720).contains(&minutes) {
        return Err("Snooze minutes must be between 5 and 720".to_string());
    }
    let until = (Local::now() + chrono::Duration::minutes(minutes as i64))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE reminders SET snoozed_until = ?1 WHERE id = ?2",
        params![until, reminder_id],
    )
    .map_err(|e| e.to_string())?;
    crate::scheduler::log_notification_history(
        &conn,
        Some(&reminder_id),
        "Reminder snoozed",
        &format!("Snoozed for {minutes} minutes until {until}"),
        "snoozed",
    )?;
    load_reminder(&conn, &reminder_id)
}

#[tauri::command]
pub fn cmd_apply_autostart(app: AppHandle, enabled: bool) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    let autolaunch = app.autolaunch();
    if enabled {
        autolaunch
            .enable()
            .map_err(|e| format!("Failed to enable Windows startup: {e}"))?;
    } else {
        autolaunch
            .disable()
            .map_err(|e| format!("Failed to disable Windows startup: {e}"))?;
    }
    Ok(autolaunch.is_enabled().unwrap_or(enabled))
}

#[tauri::command]
pub fn cmd_award_quest_xp(
    state: State<'_, AppState>,
    task_id: String,
    xp_amount: i32,
    skill_category: String,
    description: String,
    is_custom_micro_task: bool,
) -> Result<XpAwardResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let idempotency_key = format!("task_complete::{}", task_id);
    let res = award_xp_idempotent(
        &conn,
        DEFAULT_USER_ID,
        &idempotency_key,
        "task_completion",
        &task_id,
        xp_amount,
        Some(&skill_category),
        &description,
        is_custom_micro_task,
    )?;
    let today = Local::now().date_naive();
    let _ = record_productive_day(&conn, DEFAULT_USER_ID, today)?;
    Ok(res)
}

#[tauri::command]
pub fn cmd_unlock_dependent_tasks(
    state: State<'_, AppState>,
    episode_id: String,
) -> Result<usize, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    recalculate_task_dependencies(&conn, &episode_id)
}

#[tauri::command]
pub fn cmd_record_streak(state: State<'_, AppState>) -> Result<StreakRecord, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let today = Local::now().date_naive();
    let record = record_productive_day(&conn, DEFAULT_USER_ID, today)?;

    // Evaluate streak-based achievements after the streak is updated.
    let unlocks = evaluate_achievements(&conn, DEFAULT_USER_ID, AchievementNotificationMode::Individual)?;
    let _ = record_achievement_notifications(&conn, DEFAULT_USER_ID, &unlocks, AchievementNotificationMode::Individual)?;

    Ok(record)
}

#[tauri::command]
pub fn cmd_generate_ai_roadmap(input: StoryPlannerInput) -> Result<GeneratedRoadmap, String> {
    Ok(build_progressive_production_roadmap(&input))
}

#[tauri::command]
pub fn cmd_scaffold_campaign_folders(
    state: State<'_, AppState>,
    campaign_title: String,
) -> Result<String, String> {
    let root = state.app_dir.join("StoryQuestProjects");
    let path = initialize_campaign_directories(&root, &campaign_title)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn cmd_open_project_folder(folder_path: String) -> Result<(), String> {
    open_folder_in_explorer(&PathBuf::from(folder_path))
}

#[tauri::command]
pub fn cmd_save_api_key(credential_key_name: String, secret_key: String) -> Result<(), String> {
    store_api_key(&credential_key_name, &secret_key)
}

#[tauri::command]
pub fn cmd_has_api_key(credential_key_name: String) -> Result<bool, String> {
    Ok(retrieve_api_key(&credential_key_name)?.is_some())
}

#[tauri::command]
pub fn cmd_remove_api_key(credential_key_name: String) -> Result<(), String> {
    delete_api_key(&credential_key_name)
}

#[tauri::command]
pub fn cmd_trigger_test_toast(
    state: State<'_, AppState>,
    quest_title: String,
    short_description: String,
    estimated_mins: i32,
    xp_reward: i32,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    if crate::scheduler::is_suppressed_by_quiet_hours(&conn, Local::now().time()) {
        crate::scheduler::log_notification_history(
            &conn,
            None,
            &quest_title,
            &short_description,
            "quiet_hours_suppressed",
        )?;
        return Err("Suppressed by quiet hours (logged to notification history)".to_string());
    }
    dispatch_windows_toast(&quest_title, &short_description, estimated_mins, xp_reward)
}

#[tauri::command]
pub fn cmd_backup_database(state: State<'_, AppState>) -> Result<String, String> {
    let backup_dir = state.app_dir.join("Backups");
    let path = create_database_backup(&state.db_path, &backup_dir)?;
    Ok(path.to_string_lossy().to_string())
}

// ---------------------------------------------------------------------------
// M4: YouTube OAuth + sync (tokens in Windows Credential Manager only)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeClientCredentialsInput {
    pub client_id: String,
    pub client_secret: String,
}

#[tauri::command]
pub fn cmd_youtube_set_client_credentials(input: YoutubeClientCredentialsInput) -> Result<(), String> {
    crate::youtube::set_client_credentials(&input.client_id, &input.client_secret)
}

#[tauri::command]
pub fn cmd_youtube_has_client_credentials() -> bool {
    crate::youtube::has_client_credentials()
}

fn open_browser_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // rundll32 FileProtocolHandler takes the URL as a single argv entry,
        // avoiding cmd.exe splitting on '&' which truncated Google's auth URL
        // (missing response_type -> invalid_request).
        std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", url])
            .spawn()
            .map_err(|e| format!("Could not open browser: {e}"))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
    Ok(())
}

#[tauri::command]
pub async fn cmd_youtube_connect(app: AppHandle, login_hint: Option<String>) -> Result<YoutubeStatus, String> {
    use tauri::Manager;
    let (client_id, _secret) = crate::youtube::get_client_credentials()?;
    let code_verifier = crate::youtube::generate_code_verifier();
    let code_challenge = crate::youtube::pkce_challenge(&code_verifier);
    let state_token = crate::youtube::generate_state();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("Could not start local OAuth listener: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| e.to_string())?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{port}");
    let hint = login_hint
        .as_deref()
        .map(str::trim)
        .filter(|h| !h.is_empty())
        .map(|h| h.to_string())
        .or_else(crate::youtube::env_login_hint);
    let auth_url = crate::youtube::build_auth_url_with_hint(
        &client_id,
        &redirect_uri,
        &code_challenge,
        &state_token,
        hint.as_deref(),
    );

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
    }
    open_browser_url(&auth_url)?;

    let code = timeout_loopback(&listener, &state_token).await?;
    let tokens = crate::youtube::exchange_code(&code, &code_verifier, &redirect_uri).await?;
    if tokens.scope.contains("youtube.readonly") == false {
        crate::youtube::clear_tokens();
        return Err("Granted scopes did not include YouTube access; reconnect and approve all requested scopes".to_string());
    }
    let state = app.state::<AppState>();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE settings SET value = 'connected' WHERE key = 'youtube_connection_status'",
        [],
    )
    .map_err(|e| e.to_string())?;
    query_youtube_status(&conn)
}

async fn timeout_loopback(listener: &tokio::net::TcpListener, expected_state: &str) -> Result<String, String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let accepted = tokio::time::timeout(std::time::Duration::from_secs(300), listener.accept())
        .await
        .map_err(|_| "Timed out waiting for Google sign-in (5 min). Try connecting again.".to_string())?
        .map_err(|e| format!("OAuth listener error: {e}"))?;
    let (mut socket, _) = accepted;
    let mut buf = [0u8; 8192];
    let n = socket.read(&mut buf).await.map_err(|e| e.to_string())?;
    let request = String::from_utf8_lossy(&buf[..n]);
    let path = request
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap_or("/");
    let mut code: Option<String> = None;
    let mut error: Option<String> = None;
    let mut state_ok = false;
    if let Some(q) = path.split('?').nth(1) {
        for pair in q.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                let vv = urlencoding::decode(v).unwrap_or_default().to_string();
                if k == "code" {
                    code = Some(vv);
                } else if k == "error" {
                    error = Some(vv);
                } else if k == "state" && vv == expected_state {
                    state_ok = true;
                }
            }
        }
    }
    let (status, body) = if code.is_some() && state_ok {
        ("HTTP/1.1 200 OK", "YouTube connected. You can return to StoryQuest Studio.")
    } else {
        ("HTTP/1.1 400 Bad Request", "Sign-in did not complete. Return to StoryQuest Studio and try again.")
    };
    let response = format!("{status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
    let _ = socket.write_all(response.as_bytes()).await;
    if let Some(e) = error {
        return Err(format!("Google sign-in error: {e}"));
    }
    code.ok_or_else(|| "No authorization code received from Google".to_string())
}

#[tauri::command]
pub async fn cmd_youtube_sync_now(state: State<'_, AppState>) -> Result<YoutubeStatus, String> {
    let access_token = crate::youtube::ensure_access_token().await?;
    let channel = crate::youtube::fetch_channel_stats(&access_token).await?;
    let analytics = crate::youtube::fetch_analytics(&access_token).await?;
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO youtube_channel_snapshots (id, user_id, channel_id, channel_title, subscriber_count, hidden_subscriber_count, view_count, video_count, synced_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            DEFAULT_USER_ID,
            channel.channel_id,
            channel.title,
            channel.subscriber_count,
            if channel.hidden_subscriber_count { 1 } else { 0 },
            channel.view_count,
            channel.video_count,
            now,
        ],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO youtube_analytics_snapshots (id, user_id, period_start, period_end, views, estimated_minutes_watched, subscribers_gained, subscribers_lost, synced_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            DEFAULT_USER_ID,
            analytics.period_start,
            analytics.period_end,
            analytics.views,
            analytics.estimated_minutes_watched,
            analytics.subscribers_gained,
            analytics.subscribers_lost,
            now,
        ],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE creator_progress SET published_videos_total = ?1, subscriber_count = ?2, total_channel_views = ?3, total_watch_minutes = ?4, last_recalculated_at = ?5, updated_at = ?5 WHERE user_id = ?6",
        rusqlite::params![
            channel.video_count as i32,
            channel.subscriber_count,
            channel.view_count,
            analytics.estimated_minutes_watched,
            now,
            DEFAULT_USER_ID,
        ],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE settings SET value = ?1 WHERE key = 'youtube_channel_title'",
        rusqlite::params![channel.title],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE settings SET value = ?1 WHERE key = 'youtube_channel_id'",
        rusqlite::params![channel.channel_id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE settings SET value = ?1 WHERE key = 'youtube_last_sync_at'",
        rusqlite::params![now],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE settings SET value = 'connected' WHERE key = 'youtube_connection_status'",
        [],
    )
    .map_err(|e| e.to_string())?;
    let _ = crate::gamification::recalculate_creator_progress(&conn, DEFAULT_USER_ID);
    let unlocks = evaluate_achievements(&conn, DEFAULT_USER_ID, AchievementNotificationMode::Individual)?;
    let _ = record_achievement_notifications(&conn, DEFAULT_USER_ID, &unlocks, AchievementNotificationMode::Individual);
    query_youtube_status(&conn)
}

#[tauri::command]
pub fn cmd_youtube_disconnect(state: State<'_, AppState>) -> Result<YoutubeStatus, String> {
    crate::youtube::clear_tokens();
    crate::credentials::delete_api_key("youtube_oauth_client_id")?;
    crate::credentials::delete_api_key("youtube_oauth_client_secret")?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE settings SET value = 'not_connected' WHERE key = 'youtube_connection_status'",
        [],
    )
    .map_err(|e| e.to_string())?;
    query_youtube_status(&conn)
}

// ---------------------------------------------------------------------------
// M2 input structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCampaignInput {
    pub title: String,
    pub synopsis: String,
    pub genre: Option<String>,
    pub visual_style: Option<String>,
    pub target_runtime_minutes: Option<i32>,
    pub scaffold_folders: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCampaignInput {
    pub id: String,
    pub title: Option<String>,
    pub synopsis: Option<String>,
    pub genre: Option<String>,
    pub visual_style: Option<String>,
    pub target_runtime_minutes: Option<i32>,
    pub status: Option<String>,
    pub project_folder_path: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveCampaignInput {
    pub id: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEpisodeInput {
    pub campaign_id: String,
    pub title: String,
    pub premise: Option<String>,
    pub episode_number: Option<i32>,
    pub target_duration_sec: Option<i32>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEpisodeInput {
    pub id: String,
    pub title: Option<String>,
    pub premise: Option<String>,
    pub narrative_structure: Option<String>,
    pub target_duration_sec: Option<i32>,
    pub current_stage: Option<i32>,
    pub roadmap_approved: Option<bool>,
    pub status: Option<String>,
    pub youtube_url: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTaskStatusInput {
    pub task_id: String,
    pub status: String,
    pub notes: Option<String>,
    pub attached_files_json: Option<String>,
    pub actual_effort_mins: Option<i32>,
    pub completion_artifact_value: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReopenTaskInput {
    pub task_id: String,
}

// ---------------------------------------------------------------------------
// M2 helpers
// ---------------------------------------------------------------------------

pub(crate) fn query_task(conn: &Connection, task_id: &str) -> Result<Option<Task>, String> {
    conn.query_row(
        format!("SELECT {TASK_COLUMNS} FROM tasks WHERE id = ?1").as_str(),
        params![task_id],
        row_to_task,
    )
    .optional()
    .map_err(|e| format!("Failed to read task: {e}"))
}

pub(crate) fn deps_satisfied(conn: &Connection, task_id: &str) -> Result<bool, String> {
    let mut stmt = conn
        .prepare(
            "SELECT d.require_approval, t.status
             FROM task_dependencies d
             JOIN tasks t ON t.id = d.depends_on_task_id
             WHERE d.task_id = ?1",
        )
        .map_err(|e| e.to_string())?;
    let deps: Vec<(i32, String)> = stmt
        .query_map(params![task_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .collect();
    Ok(deps.iter().all(|(req_approval, dep_status)| {
        if *req_approval != 0 {
            dep_status == "approved" || dep_status == "completed"
        } else {
            dep_status == "completed" || dep_status == "approved"
        }
    }))
}

fn valid_transition(current: &str, target: &str) -> bool {
    matches!((current, target),
        ("available", "in_progress") |
        ("available", "paused") |
        ("in_progress", "available") |
        ("in_progress", "paused") |
        ("in_progress", "review") |
        ("paused", "in_progress") |
        ("paused", "available") |
        ("review", "in_progress") |
        ("review", "approved") |
        ("approved", "completed") |
        ("completed", "available") |
        ("locked", "available"))
}

#[tauri::command]
pub fn cmd_create_campaign(
    state: State<'_, AppState>,
    input: CreateCampaignInput,
) -> Result<Campaign, String> {
    let title = input.title.trim();
    let synopsis = input.synopsis.trim();
    if title.is_empty() {
        return Err("Campaign title cannot be empty.".to_string());
    }
    if synopsis.is_empty() {
        return Err("Campaign synopsis cannot be empty.".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let genre = input.genre.unwrap_or_else(|| "Dark Fantasy".to_string());
    let visual_style = input.visual_style.unwrap_or_else(|| "2.5D Stylized Cel-Shaded".to_string());
    let target_runtime_minutes = input.target_runtime_minutes.unwrap_or(8);
    let scaffold = input.scaffold_folders.unwrap_or(false);

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO campaigns (id, title, synopsis, genre, visual_style, target_runtime_minutes,
                 status, project_folder_path, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'planning', ?7, ?8, ?8)",
        params![
            id,
            title,
            synopsis,
            genre,
            visual_style,
            target_runtime_minutes,
            state.app_dir.join("StoryQuestProjects").join(title).to_string_lossy(),
            now
        ],
    )
    .map_err(|e| format!("Failed to create campaign: {e}"))?;

    if scaffold {
        let _ = initialize_campaign_directories(
            &state.app_dir.join("StoryQuestProjects"),
            title,
        );
    }

    query_campaign(&conn, &id)?
        .ok_or_else(|| "Campaign created but could not be read back.".to_string())
}

#[tauri::command]
pub fn cmd_update_campaign(
    state: State<'_, AppState>,
    input: UpdateCampaignInput,
) -> Result<Campaign, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    if let Some(title) = &input.title {
        if title.trim().is_empty() {
            return Err("Campaign title cannot be empty.".to_string());
        }
        conn.execute(
            "UPDATE campaigns SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![title.trim(), now, input.id],
        )
        .map_err(|e| format!("Failed to update campaign title: {e}"))?;
    }
    if let Some(synopsis) = &input.synopsis {
        conn.execute(
            "UPDATE campaigns SET synopsis = ?1, updated_at = ?2 WHERE id = ?3",
            params![synopsis.trim(), now, input.id],
        )
        .map_err(|e| format!("Failed to update campaign synopsis: {e}"))?;
    }
    if let Some(genre) = &input.genre {
        conn.execute(
            "UPDATE campaigns SET genre = ?1, updated_at = ?2 WHERE id = ?3",
            params![genre, now, input.id],
        )
        .map_err(|e| format!("Failed to update campaign genre: {e}"))?;
    }
    if let Some(visual_style) = &input.visual_style {
        conn.execute(
            "UPDATE campaigns SET visual_style = ?1, updated_at = ?2 WHERE id = ?3",
            params![visual_style, now, input.id],
        )
        .map_err(|e| format!("Failed to update campaign visual style: {e}"))?;
    }
    if let Some(target_runtime_minutes) = input.target_runtime_minutes {
        conn.execute(
            "UPDATE campaigns SET target_runtime_minutes = ?1, updated_at = ?2 WHERE id = ?3",
            params![target_runtime_minutes, now, input.id],
        )
        .map_err(|e| format!("Failed to update campaign target runtime: {e}"))?;
    }
    if let Some(status) = &input.status {
        conn.execute(
            "UPDATE campaigns SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, now, input.id],
        )
        .map_err(|e| format!("Failed to update campaign status: {e}"))?;
    }
    if let Some(project_folder_path) = &input.project_folder_path {
        conn.execute(
            "UPDATE campaigns SET project_folder_path = ?1, updated_at = ?2 WHERE id = ?3",
            params![project_folder_path, now, input.id],
        )
        .map_err(|e| format!("Failed to update campaign folder path: {e}"))?;
    }

    query_campaign(&conn, &input.id)?
        .ok_or_else(|| "Campaign not found.".to_string())
}

#[tauri::command]
pub fn cmd_archive_campaign(
    state: State<'_, AppState>,
    input: ArchiveCampaignInput,
) -> Result<Campaign, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "UPDATE campaigns SET status = 'archived', updated_at = ?1 WHERE id = ?2",
        params![now, input.id],
    )
    .map_err(|e| format!("Failed to archive campaign: {e}"))?;
    query_campaign(&conn, &input.id)?
        .ok_or_else(|| "Campaign not found.".to_string())
}

#[tauri::command]
pub fn cmd_create_episode(
    state: State<'_, AppState>,
    input: CreateEpisodeInput,
) -> Result<Episode, String> {
    let title = input.title.trim();
    if title.is_empty() {
        return Err("Episode title cannot be empty.".to_string());
    }

    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Verify campaign exists.
    query_campaign(&conn, &input.campaign_id)?
        .ok_or_else(|| "Campaign not found.".to_string())?;

    let episode_number = match input.episode_number {
        Some(n) if n > 0 => n,
        _ => {
            let max: Option<i32> = conn
                .query_row(
                    "SELECT MAX(episode_number) FROM episodes WHERE campaign_id = ?1",
                    params![&input.campaign_id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|e| e.to_string())?;
            max.unwrap_or(0) + 1
        }
    };

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let target_duration_sec = input.target_duration_sec.unwrap_or(480);
    let premise = input.premise.unwrap_or_default();
    let narrative_structure = String::new();

    conn.execute(
        "INSERT INTO episodes (id, campaign_id, episode_number, title, premise, narrative_structure,
                 target_duration_sec, current_stage, roadmap_approved, status, youtube_url,
                 created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, 0, 'draft', NULL, ?8, ?8)",
        params![
            id,
            input.campaign_id,
            episode_number,
            title,
            premise,
            narrative_structure,
            target_duration_sec,
            now
        ],
    )
    .map_err(|e| format!("Failed to create episode: {e}"))?;

    query_episode(&conn, &input.campaign_id)?
        .and_then(|ep| if ep.id == id { Some(ep) } else { None })
        .ok_or_else(|| "Episode created but could not be read back.".to_string())
}

#[tauri::command]
pub fn cmd_update_episode(
    state: State<'_, AppState>,
    input: UpdateEpisodeInput,
) -> Result<Episode, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let episode_id = &input.id;

    if let Some(title) = &input.title {
        if title.trim().is_empty() {
            return Err("Episode title cannot be empty.".to_string());
        }
        conn.execute(
            "UPDATE episodes SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![title.trim(), now, episode_id],
        )
        .map_err(|e| format!("Failed to update episode title: {e}"))?;
    }
    if let Some(premise) = &input.premise {
        conn.execute(
            "UPDATE episodes SET premise = ?1, updated_at = ?2 WHERE id = ?3",
            params![premise, now, episode_id],
        )
        .map_err(|e| format!("Failed to update episode premise: {e}"))?;
    }
    if let Some(narrative_structure) = &input.narrative_structure {
        conn.execute(
            "UPDATE episodes SET narrative_structure = ?1, updated_at = ?2 WHERE id = ?3",
            params![narrative_structure, now, episode_id],
        )
        .map_err(|e| format!("Failed to update episode narrative structure: {e}"))?;
    }
    if let Some(target_duration_sec) = input.target_duration_sec {
        conn.execute(
            "UPDATE episodes SET target_duration_sec = ?1, updated_at = ?2 WHERE id = ?3",
            params![target_duration_sec, now, episode_id],
        )
        .map_err(|e| format!("Failed to update episode target duration: {e}"))?;
    }
    if let Some(current_stage) = input.current_stage {
        conn.execute(
            "UPDATE episodes SET current_stage = ?1, updated_at = ?2 WHERE id = ?3",
            params![current_stage, now, episode_id],
        )
        .map_err(|e| format!("Failed to update episode current stage: {e}"))?;
    }
    if let Some(roadmap_approved) = input.roadmap_approved {
        conn.execute(
            "UPDATE episodes SET roadmap_approved = ?1, updated_at = ?2 WHERE id = ?3",
            params![if roadmap_approved { 1 } else { 0 }, now, episode_id],
        )
        .map_err(|e| format!("Failed to update episode roadmap approval: {e}"))?;
    }
    if let Some(status) = &input.status {
        conn.execute(
            "UPDATE episodes SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, now, episode_id],
        )
        .map_err(|e| format!("Failed to update episode status: {e}"))?;
    }
    if let Some(youtube_url) = &input.youtube_url {
        conn.execute(
            "UPDATE episodes SET youtube_url = ?1, updated_at = ?2 WHERE id = ?3",
            params![youtube_url, now, episode_id],
        )
        .map_err(|e| format!("Failed to update episode YouTube URL: {e}"))?;
    }

    // query_episode takes campaign_id; fetch campaign_id first.
    let campaign_id: String = conn
        .query_row("SELECT campaign_id FROM episodes WHERE id = ?1", params![episode_id], |row| row.get(0))
        .map_err(|e| format!("Failed to read episode campaign: {e}"))?;
    query_episode(&conn, &campaign_id)?
        .and_then(|ep| if ep.id == *episode_id { Some(ep) } else { None })
        .ok_or_else(|| "Episode not found.".to_string())
}

#[tauri::command]
pub fn cmd_set_task_status(
    state: State<'_, AppState>,
    input: SetTaskStatusInput,
) -> Result<Task, String> {
    let valid_statuses = [
        "locked", "blocked", "available", "in_progress", "paused", "review", "approved", "completed",
    ];
    if !valid_statuses.contains(&input.status.as_str()) {
        return Err(format!("Invalid task status: {}", input.status));
    }

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let task = query_task(&conn, &input.task_id)?
        .ok_or_else(|| "Task not found.".to_string())?;

    if !valid_transition(&task.status, &input.status) {
        return Err(format!(
            "Cannot transition task from '{}' to '{}'.",
            task.status, input.status
        ));
    }

    // Completing a task requires all mandatory dependencies to be satisfied.
    if input.status == "completed" {
        if !deps_satisfied(&conn, &input.task_id)? {
            return Err("Cannot complete task: not all dependencies are satisfied.".to_string());
        }
    }

    let mut set_clauses = Vec::new();
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    set_clauses.push("status = ?".to_string());
    params_vec.push(Box::new(input.status.clone()));
    set_clauses.push("updated_at = ?".to_string());
    params_vec.push(Box::new(now.clone()));

    if input.status == "completed" {
        set_clauses.push("completed_at = ?".to_string());
        params_vec.push(Box::new(now.clone()));
    } else if input.status != "completed" && task.completed_at.is_some() {
        set_clauses.push("completed_at = NULL".to_string());
    }

    if let Some(notes) = &input.notes {
        set_clauses.push("notes = ?".to_string());
        params_vec.push(Box::new(notes.clone()));
    }
    if let Some(attached_files_json) = &input.attached_files_json {
        set_clauses.push("attached_files_json = ?".to_string());
        params_vec.push(Box::new(attached_files_json.clone()));
    }
    if let Some(actual_effort_mins) = input.actual_effort_mins {
        set_clauses.push("actual_effort_mins = ?".to_string());
        params_vec.push(Box::new(actual_effort_mins));
    }
    if let Some(completion_artifact_value) = &input.completion_artifact_value {
        set_clauses.push("completion_artifact_value = ?".to_string());
        params_vec.push(Box::new(completion_artifact_value.clone()));
    }

    if !set_clauses.is_empty() {
        let sql = format!("UPDATE tasks SET {} WHERE id = ?", set_clauses.join(", "));
        params_vec.push(Box::new(input.task_id.clone()));
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        conn.execute(&sql, params_refs.as_slice())
            .map_err(|e| format!("Failed to update task status: {e}"))?;
    }

    // Award XP when a task is completed (idempotent).
    if input.status == "completed" {
        let idempotency_key = format!("task_complete::{}", input.task_id);
        let _ = award_xp_idempotent(
            &conn,
            DEFAULT_USER_ID,
            &idempotency_key,
            "task_completion",
            &input.task_id,
            task.xp_reward,
            Some(&task.skill_category),
            &format!("Completed task: {}", task.title),
            false,
        );
        let today = Local::now().date_naive();
        let _ = record_productive_day(&conn, DEFAULT_USER_ID, today);

        // Evaluate achievements and surface any unlocks via durable notifications.
        let unlocks = evaluate_achievements(&conn, DEFAULT_USER_ID, AchievementNotificationMode::Individual)?;
        let _ = record_achievement_notifications(&conn, DEFAULT_USER_ID, &unlocks, AchievementNotificationMode::Individual)?;
    }

    // Unlock dependent tasks now that this one may be done.
    let _ = recalculate_task_dependencies(&conn, &task.episode_id);

    let mut updated = query_task(&conn, &input.task_id)?
        .ok_or_else(|| "Task not found after update.".to_string())?;
    updated.dependencies = load_dependencies(&conn, &input.task_id)?;
    Ok(updated)
}

#[tauri::command]
pub fn cmd_reopen_task(
    state: State<'_, AppState>,
    input: ReopenTaskInput,
) -> Result<Task, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let task = query_task(&conn, &input.task_id)?
        .ok_or_else(|| "Task not found.".to_string())?;

    if task.status != "completed" {
        return Err("Only completed tasks can be reopened.".to_string());
    }

    conn.execute(
        "UPDATE tasks SET status = 'available', completed_at = NULL, updated_at = ?1 WHERE id = ?2",
        params![now, input.task_id],
    )
    .map_err(|e| format!("Failed to reopen task: {e}"))?;

    // Re-evaluate dependency locking in case reopening should relock dependents.
    let _ = recalculate_task_dependencies(&conn, &task.episode_id);

    let mut updated = query_task(&conn, &input.task_id)?
        .ok_or_else(|| "Task not found after reopen.".to_string())?;
    updated.dependencies = load_dependencies(&conn, &input.task_id)?;
    Ok(updated)
}

#[tauri::command]
pub fn cmd_get_achievement_notifications(
    state: State<'_, AppState>,
) -> Result<Vec<AchievementNotification>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, achievement_id, kind, title, body, items_json,
                     presented_at, acknowledged_at
              FROM achievement_notifications
              WHERE user_id = ?1 AND acknowledged_at IS NULL
              ORDER BY created_at ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![DEFAULT_USER_ID], |row| {
            Ok(AchievementNotification {
                id: row.get(0)?,
                achievement_id: row.get(1)?,
                kind: row.get(2)?,
                title: row.get(3)?,
                body: row.get(4)?,
                items_json: row.get(5)?,
                presented_at: row.get(6)?,
                acknowledged_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_acknowledge_achievement_notification(
    state: State<'_, AppState>,
    input: AcknowledgeNotificationsInput,
) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    if input.notification_ids.is_empty() {
        return Ok(true);
    }

    let placeholders: Vec<String> = (0..input.notification_ids.len()).map(|_| "?".to_string()).collect();
    let sql = format!(
        "UPDATE achievement_notifications SET acknowledged_at = datetime('now')
         WHERE user_id = ? AND acknowledged_at IS NULL AND id IN ({})",
        placeholders.join(", ")
    );
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(DEFAULT_USER_ID.to_string())];
    for id in &input.notification_ids {
        params.push(Box::new(id.clone()));
    }
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    conn.execute(&sql, param_refs.as_slice())
        .map_err(|e| format!("Failed to acknowledge notifications: {e}"))?;
    Ok(true)
}

#[tauri::command]
pub fn cmd_set_streak_prefs(
    state: State<'_, AppState>,
    input: SetStreakPrefsInput,
) -> Result<StreakRecord, String> {
    if input.weekly_rest_day < 0 || input.weekly_rest_day > 6 {
        return Err("weekly_rest_day must be between 0 and 6.".to_string());
    }
    if let Some(until) = &input.vacation_until {
        if NaiveDate::parse_from_str(until, "%Y-%m-%d").is_err() {
            return Err("vacation_until must be YYYY-MM-DD.".to_string());
        }
    }

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.execute(
        "INSERT INTO streak_records (id, user_id, current_streak, best_streak,
                 streak_freezes_available, weekly_rest_day, vacation_mode, vacation_until,
                 last_productive_date, history_json, updated_at)
         VALUES ('streak_user_solo_01', ?1, 0, 0, 2, ?2, ?3, ?4, NULL, '[]', ?5)
         ON CONFLICT(user_id) DO UPDATE SET
             weekly_rest_day = ?2,
             vacation_mode = ?3,
             vacation_until = ?4,
             updated_at = ?5",
        params![
            DEFAULT_USER_ID,
            input.weekly_rest_day,
            if input.vacation_mode { 1 } else { 0 },
            input.vacation_until,
            now
        ],
    )
    .map_err(|e| format!("Failed to save streak preferences: {e}"))?;

    query_streak(&conn, DEFAULT_USER_ID)
}
