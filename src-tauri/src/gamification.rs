use chrono::{Datelike, NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::models::{StreakRecord, XpAwardResult, PRODUCTION_STAGES};

pub const DAILY_CUSTOM_TASK_XP_CAP: i32 = 150;

pub fn calculate_level_from_xp(conn: &Connection, total_xp: i32) -> Result<(i32, String), String> {
    let mut stmt = conn
        .prepare("SELECT level, title FROM levels WHERE min_xp <= ?1 ORDER BY level DESC LIMIT 1")
        .map_err(|e| e.to_string())?;
    let result = stmt
        .query_row(params![total_xp], |row| Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?)))
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(result.unwrap_or((1, "Apprentice".to_string())))
}

/// Awards XP idempotently using `idempotency_key` so completing, reopening, and re-completing
/// the same task or milestone can never duplicate XP. Also enforces daily XP caps on custom micro-tasks.
pub fn award_xp_idempotent(
    conn: &Connection,
    user_id: &str,
    idempotency_key: &str,
    source_type: &str,
    source_id: &str,
    requested_xp: i32,
    skill_category: Option<&str>,
    description: &str,
    is_custom_micro_task: bool,
) -> Result<XpAwardResult, String> {
    // 1. Check if idempotency_key already exists in xp_events
    let existing: Option<i32> = conn
        .query_row(
            "SELECT xp_amount FROM xp_events WHERE idempotency_key = ?1",
            params![idempotency_key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let (old_level, current_xp, current_title): (i32, i32, String) = conn
        .query_row(
            "SELECT current_level, total_xp, active_title FROM users WHERE id = ?1",
            params![user_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| e.to_string())?;

    if existing.is_some() {
        return Ok(XpAwardResult {
            awarded: false,
            xp_delta: 0,
            new_total_xp: current_xp,
            old_level,
            new_level: old_level,
            new_title: current_title,
            unlocked_achievements: vec![],
            reason: "XP already awarded for this milestone/quest ledger key.".to_string(),
        });
    }

    // 2. Enforce anti-exploit daily cap on repeated tiny custom tasks
    let today_str = Utc::now().date_naive().to_string();
    let mut final_xp = requested_xp.max(0);
    if is_custom_micro_task {
        let (daily_used, last_date): (i32, Option<String>) = conn
            .query_row(
                "SELECT daily_xp_custom_tasks, last_custom_xp_date FROM users WHERE id = ?1",
                params![user_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| e.to_string())?;

        let used_today = if last_date.as_deref() == Some(today_str.as_str()) {
            daily_used
        } else {
            0
        };

        if used_today >= DAILY_CUSTOM_TASK_XP_CAP {
            return Ok(XpAwardResult {
                awarded: false,
                xp_delta: 0,
                new_total_xp: current_xp,
                old_level,
                new_level: old_level,
                new_title: current_title,
                unlocked_achievements: vec![],
                reason: "Daily custom micro-task XP cap reached. Complete main campaign quests for full XP.".to_string(),
            });
        }
        final_xp = final_xp.min(DAILY_CUSTOM_TASK_XP_CAP - used_today);
        conn.execute(
            "UPDATE users SET daily_xp_custom_tasks = ?1, last_custom_xp_date = ?2 WHERE id = ?3",
            params![used_today + final_xp, today_str, user_id],
        )
        .map_err(|e| e.to_string())?;
    }

    // 3. Insert XP event into ledger
    let event_id = format!("xp_{}", Uuid::new_v4());
    conn.execute(
        "INSERT INTO xp_events (id, user_id, idempotency_key, source_type, source_id, xp_amount, skill_category, description)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            event_id,
            user_id,
            idempotency_key,
            source_type,
            source_id,
            final_xp,
            skill_category,
            description
        ],
    )
    .map_err(|e| e.to_string())?;

    // 4. Update User XP & Level
    let new_total_xp = current_xp + final_xp;
    let (new_level, new_title) = calculate_level_from_xp(conn, new_total_xp)?;
    conn.execute(
        "UPDATE users SET total_xp = ?1, current_level = ?2, active_title = ?3, updated_at = datetime('now') WHERE id = ?4",
        params![new_total_xp, new_level, new_title, user_id],
    )
    .map_err(|e| e.to_string())?;

    // 5. Update Skill Tree Category
    if let Some(cat) = skill_category {
        conn.execute(
            "UPDATE skill_categories
             SET xp_earned = xp_earned + ?1,
                 tasks_completed = tasks_completed + 1,
                 rank_level = 1 + ((xp_earned + ?1) / 100)
             WHERE category = ?2",
            params![final_xp, cat],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(XpAwardResult {
        awarded: true,
        xp_delta: final_xp,
        new_total_xp,
        old_level,
        new_level,
        new_title,
        unlocked_achievements: vec![],
        reason: "XP awarded and recorded in ledger.".to_string(),
    })
}

/// Recomputes locked -> available transitions for tasks whose dependencies are satisfied.
pub fn recalculate_task_dependencies(conn: &Connection, episode_id: &str) -> Result<usize, String> {
    let mut stmt = conn
        .prepare("SELECT id, status FROM tasks WHERE episode_id = ?1")
        .map_err(|e| e.to_string())?;
    let tasks: Vec<(String, String)> = stmt
        .query_map(params![episode_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .collect();

    let mut unlocked_count = 0;
    for (task_id, status) in tasks {
        if status != "locked" && status != "blocked" {
            continue;
        }
        let mut dep_stmt = conn
            .prepare(
                "SELECT d.require_approval, t.status
                 FROM task_dependencies d
                 JOIN tasks t ON t.id = d.depends_on_task_id
                 WHERE d.task_id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let deps: Vec<(i32, String)> = dep_stmt
            .query_map(params![&task_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .collect();

        let all_satisfied = deps.iter().all(|(req_approval, dep_status)| {
            if *req_approval != 0 {
                dep_status == "approved" || dep_status == "completed"
            } else {
                dep_status == "completed" || dep_status == "approved"
            }
        });

        if all_satisfied && status == "locked" {
            conn.execute(
                "UPDATE tasks SET status = 'available', updated_at = datetime('now') WHERE id = ?1",
                params![&task_id],
            )
            .map_err(|e| e.to_string())?;
            unlocked_count += 1;
        }
    }
    Ok(unlocked_count)
}

pub fn list_creator_ranks(conn: &Connection) -> Result<Vec<crate::models::CreatorRank>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT level, rank_name, min_published_videos, badge_icon, description
             FROM creator_ranks ORDER BY level ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(crate::models::CreatorRank {
                level: row.get(0)?,
                rank_name: row.get(1)?,
                min_published_videos: row.get(2)?,
                badge_icon: row.get(3)?,
                description: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn creator_rank_for_published(
    conn: &Connection,
    published_videos: i32,
) -> Result<crate::models::CreatorRank, String> {
    let mut stmt = conn
        .prepare(
            "SELECT level, rank_name, min_published_videos, badge_icon, description
             FROM creator_ranks
             WHERE min_published_videos <= ?1
             ORDER BY min_published_videos DESC LIMIT 1",
        )
        .map_err(|e| e.to_string())?;
    stmt.query_row(params![published_videos], |row| {
        Ok(crate::models::CreatorRank {
            level: row.get(0)?,
            rank_name: row.get(1)?,
            min_published_videos: row.get(2)?,
            badge_icon: row.get(3)?,
            description: row.get(4)?,
        })
    })
    .optional()
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "No creator rank is configured for this published video count.".to_string())
}

pub fn get_creator_progress(
    conn: &Connection,
    user_id: &str,
) -> Result<crate::models::CreatorProgress, String> {
    conn.execute(
        "INSERT OR IGNORE INTO creator_progress (user_id, rank_level, published_videos_total) VALUES (?1, 1, 0)",
        params![user_id],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT user_id, rank_level, published_videos_total, shorts_count, long_form_count,
                subscriber_count, total_channel_views, total_watch_minutes,
                last_recalculated_at, updated_at
         FROM creator_progress WHERE user_id = ?1",
        params![user_id],
        |row| {
            Ok(crate::models::CreatorProgress {
                user_id: row.get(0)?,
                rank_level: row.get(1)?,
                published_videos_total: row.get(2)?,
                shorts_count: row.get(3)?,
                long_form_count: row.get(4)?,
                subscriber_count: row.get(5)?,
                total_channel_views: row.get(6)?,
                total_watch_minutes: row.get(7)?,
                last_recalculated_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

pub fn recalculate_creator_progress(
    conn: &Connection,
    user_id: &str,
) -> Result<crate::models::CreatorProgress, String> {
    let progress = get_creator_progress(conn, user_id)?;
    let rank = creator_rank_for_published(conn, progress.published_videos_total)?;
    conn.execute(
        "UPDATE creator_progress
         SET rank_level = ?1, last_recalculated_at = datetime('now'), updated_at = datetime('now')
         WHERE user_id = ?2",
        params![rank.level, user_id],
    )
    .map_err(|e| e.to_string())?;
    get_creator_progress(conn, user_id)
}

#[derive(Debug, Clone, PartialEq)]
pub enum AchievementNotificationMode {
    Individual,
    Summary,
}

impl AchievementNotificationMode {
    fn as_str(&self) -> &'static str {
        match self {
            AchievementNotificationMode::Individual => "unlock",
            AchievementNotificationMode::Summary => "backfill_summary",
        }
    }
}

#[derive(Debug, Clone)]
struct AchievementEvalRow {
    id: String,
    code: String,
    title: String,
    xp_bonus: i32,
    condition_type: String,
    condition_value: i32,
    condition_scope: String,
    progress_target: i32,
    progress_available: bool,
    progress_current: i32,
}

/// Evaluates every locked achievement against current state and persists newly unlocked ones.
///
/// Idempotent: re-running never re-awards XP, never changes `unlocked_at`, and never creates
/// duplicate notifications because `dedupe_key` is unique and achievements are only unlocked
/// once. `manual` achievements are never auto-unlocked. Hidden achievements are still evaluated
/// and unlocked; the frontend decides whether to render them while hidden.
///
/// Returns one `AchievementUnlock` per newly unlocked achievement.
pub fn evaluate_achievements(
    conn: &Connection,
    user_id: &str,
    mode: AchievementNotificationMode,
) -> Result<Vec<crate::models::AchievementUnlock>, String> {
    let progress = get_creator_progress(conn, user_id)?;
    let streak: (i32, i32) = conn
        .query_row(
            "SELECT current_streak, best_streak FROM streak_records WHERE user_id = ?1 LIMIT 1",
            params![user_id],
            |row| Ok((row.get::<_, i32>(0)?, row.get::<_, i32>(1)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .unwrap_or((0, 0));
    let consistency_weeks = streak.1 / 7;

    // Tasks currently approved/completed.
    let completed_task_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM tasks WHERE status IN ('approved', 'completed')",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .unwrap_or(0);

    // Tasks per stage that are approved/completed.
    let mut stage_stmt = conn
        .prepare(
            "SELECT stage_number, COUNT(*) FROM tasks
             WHERE status IN ('approved', 'completed')
             GROUP BY stage_number",
        )
        .map_err(|e| e.to_string())?;
    let stage_rows = stage_stmt
        .query_map([], |row| Ok((row.get::<_, i32>(0)?, row.get::<_, i32>(1)?)))
        .map_err(|e| e.to_string())?;
    let mut tasks_per_stage: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();
    for row in stage_rows {
        let (stage, count) = row.map_err(|e| e.to_string())?;
        tasks_per_stage.insert(stage, count);
    }

    // Episodes whose all canonical stages are covered by approved/completed tasks.
    let mut ep_stmt = conn
        .prepare("SELECT id, episode_number FROM episodes ORDER BY episode_number ASC")
        .map_err(|e| e.to_string())?;
    let episodes = ep_stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)))
        .map_err(|e| e.to_string())?;
    let mut completed_episode_count: i32 = 0;
    for ep in episodes {
        let (ep_id, ep_num) = ep.map_err(|e| e.to_string())?;
        let covered: Vec<i32> = conn
            .prepare(
                "SELECT DISTINCT stage_number FROM tasks
                 WHERE episode_id = ?1 AND status IN ('approved', 'completed')",
            )
            .map_err(|e| e.to_string())?
            .query_map(params![&ep_id], |row| row.get::<_, i32>(0))
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .collect();
        let all_covered = PRODUCTION_STAGES.iter().all(|(stage, _, _, _)| covered.contains(stage));
        if all_covered && ep_num > 0 {
            completed_episode_count += 1;
        }
    }

    let mut unlocked: Vec<crate::models::AchievementUnlock> = Vec::new();

    let mut stmt = conn
        .prepare(
            "SELECT id, code, title, xp_bonus, condition_type, condition_value, condition_scope,
                     progress_target, progress_available, progress_current
             FROM achievements WHERE unlocked = 0",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(AchievementEvalRow {
                id: row.get(0)?,
                code: row.get(1)?,
                title: row.get(2)?,
                xp_bonus: row.get(3)?,
                condition_type: row.get(4)?,
                condition_value: row.get(5)?,
                condition_scope: row.get(6)?,
                progress_target: row.get(7)?,
                progress_available: row.get::<_, i32>(8)? != 0,
                progress_current: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut rows = rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    for row in &mut rows {
        if row.condition_type == "manual" {
            continue;
        }

        let computed = match row.condition_type.as_str() {
            "task_completed" => completed_task_count,
            "stage_status" => {
                let scope = row.condition_scope.as_str();
                let stage = scope.strip_prefix("stage:").and_then(|s| s.parse::<i32>().ok());
                match stage {
                    Some(s) => *tasks_per_stage.get(&s).unwrap_or(&0),
                    None => 0,
                }
            }
            "episode_completed" => completed_episode_count,
            "published_count" => progress.published_videos_total,
            "channel_views" => progress.total_channel_views.unwrap_or(0) as i32,
            "subscribers" => progress.subscriber_count.unwrap_or(0) as i32,
            "watch_minutes" => progress.total_watch_minutes.unwrap_or(0) as i32,
            "consistency_weeks" => consistency_weeks,
            _ => 0,
        };

        if row.progress_available && computed >= row.progress_target {
            let new_progress = row.progress_target.min(computed);
            conn.execute(
                "UPDATE achievements
                 SET unlocked = 1, unlocked_at = datetime('now'),
                     progress_current = ?1
                 WHERE id = ?2",
                params![new_progress, row.id],
            )
            .map_err(|e| e.to_string())?;
            if row.xp_bonus > 0 {
                let idempotency_key = format!("achievement_unlock::{user_id}::{}", row.code);
                let _ = award_xp_idempotent(
                    conn,
                    user_id,
                    &idempotency_key,
                    "achievement_unlock",
                    &row.id,
                    row.xp_bonus,
                    None,
                    &format!("Achievement unlocked: {}", row.title),
                    false,
                );
            }

            unlocked.push(crate::models::AchievementUnlock {
                achievement_id: row.id.clone(),
                code: row.code.clone(),
                title: row.title.clone(),
                xp_bonus: row.xp_bonus,
            });
        } else if row.progress_available {
            let new_progress = row.progress_target.min(computed);
            if new_progress != row.progress_current {
                conn.execute(
                    "UPDATE achievements SET progress_current = ?1 WHERE id = ?2",
                    params![new_progress, row.id],
                )
                .map_err(|e| e.to_string())?;
                row.progress_current = new_progress;
            }
        }
    }

    Ok(unlocked)
}

/// Persists a durable notification row for each unlock, or one consolidated summary row.
pub fn record_achievement_notifications(
    conn: &Connection,
    user_id: &str,
    unlocks: &[crate::models::AchievementUnlock],
    mode: AchievementNotificationMode,
) -> Result<(), String> {
    if unlocks.is_empty() {
        return Ok(());
    }

    match mode {
        AchievementNotificationMode::Individual => {
            for unlock in unlocks {
                let dedupe_key = format!("unlock::{user_id}::{}", unlock.code);
                let id = format!("ntf_{}", Uuid::new_v4());
                let title = format!("Achievement unlocked: {}", unlock.title);
                let body = if unlock.xp_bonus > 0 {
                    format!("+{} XP — {}", unlock.xp_bonus, unlock.title)
                } else {
                    unlock.title.clone()
                };
                let items_json = serde_json::to_string(&vec![unlock.title.as_str()])
                    .unwrap_or_else(|_| "[]".to_string());

                let existing: Option<String> = conn
                    .query_row(
                        "SELECT id FROM achievement_notifications WHERE dedupe_key = ?1",
                        params![dedupe_key],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(|e| e.to_string())?;
                if existing.is_some() {
                    continue;
                }

                conn.execute(
                    "INSERT INTO achievement_notifications
                     (id, user_id, achievement_id, kind, title, body, items_json, dedupe_key)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        id,
                        user_id,
                        unlock.achievement_id,
                        AchievementNotificationMode::Individual.as_str(),
                        title,
                        body,
                        items_json,
                    dedupe_key,
                ],
            )
            .map_err(|e| e.to_string())?;
            }
        }
        AchievementNotificationMode::Summary => {
            let titles: Vec<String> = unlocks.iter().map(|u| u.title.clone()).collect();
            let items_json = serde_json::to_string(&titles).unwrap_or_else(|_| "[]".to_string());
            let codes: Vec<String> = unlocks.iter().map(|u| u.code.clone()).collect();
            let dedupe_key = format!("backfill::{user_id}::{}", codes.join(","));
            let id = format!("ntf_{}", Uuid::new_v4());
            let count = unlocks.len();
            let title = format!("{count} achievements unlocked");
            let body = titles.join(", ");

            let existing: Option<String> = conn
                .query_row(
                    "SELECT id FROM achievement_notifications WHERE dedupe_key = ?1",
                    params![dedupe_key],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|e| e.to_string())?;
            if existing.is_some() {
                return Ok(());
            }

            conn.execute(
                "INSERT INTO achievement_notifications
                 (id, user_id, achievement_id, kind, title, body, items_json, dedupe_key)
                 VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7)",
                params![
                    id,
                    user_id,
                    AchievementNotificationMode::Summary.as_str(),
                    title,
                    body,
                    items_json,
                    dedupe_key,
                ],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

/// Evaluates and updates the user's daily streak without ever deducting XP for inactivity.
///
/// Streak rules:
/// - A productive day today extends the streak if no eligible days were missed since the last
///   productive day.
/// - Rest days (`weekly_rest_day`) and active vacation days are excluded from eligible missed
///   days and never break the streak.
/// - One or two eligible missed days consume one freeze per day. Without enough freezes, the
///   streak resets to 1.
/// - More than two eligible missed days resets the streak to 1, regardless of available freezes.
/// - `vacation_until` is inclusive; a productive day on or before that date is protected.
/// - Vacation expires on the day after `vacation_until`.
pub fn record_productive_day(conn: &Connection, user_id: &str, today: NaiveDate) -> Result<StreakRecord, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, user_id, current_streak, best_streak, streak_freezes_available,
                    weekly_rest_day, vacation_mode, vacation_until, last_productive_date, history_json
             FROM streak_records WHERE user_id = ?1 LIMIT 1",
        )
        .map_err(|e| e.to_string())?;

    let mut record = stmt
        .query_row(params![user_id], |row| {
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
        })
        .map_err(|e| e.to_string())?;

    let today_str = today.to_string();
    if record.last_productive_date.as_deref() == Some(today_str.as_str()) {
        return Ok(record);
    }

    // Expire vacation inclusively: vacation protects through vacation_until, expires the next day.
    if let Some(until) = &record.vacation_until {
        if let Ok(until_date) = NaiveDate::parse_from_str(until, "%Y-%m-%d") {
            if today > until_date {
                record.vacation_mode = false;
                record.vacation_until = None;
            }
        }
    }

    if let Some(last_str) = &record.last_productive_date {
        if let Ok(last_date) = NaiveDate::parse_from_str(last_str, "%Y-%m-%d") {
            let mut eligible_missed = 0i32;
            let mut day = last_date.succ_opt().unwrap_or(last_date);
            while day < today {
                let weekday = day.weekday().num_days_from_sunday() as i32;
                let is_rest = weekday == record.weekly_rest_day;
                let is_vacation = record.vacation_mode
                    && record.vacation_until.as_deref().map_or(false, |until| {
                        NaiveDate::parse_from_str(until, "%Y-%m-%d")
                            .map_or(false, |d| day <= d)
                    });
                if !is_rest && !is_vacation {
                    eligible_missed += 1;
                }
                day = day.succ_opt().unwrap_or(day);
            }

            if eligible_missed == 0 {
                record.current_streak += 1;
            } else if eligible_missed <= 2 {
                if record.streak_freezes_available >= eligible_missed {
                    record.streak_freezes_available -= eligible_missed;
                    record.current_streak += 1;
                } else {
                    record.current_streak = 1;
                }
            } else {
                record.current_streak = 1;
            }
        } else {
            record.current_streak = 1;
        }
    } else {
        record.current_streak = 1;
    }

    if record.current_streak > record.best_streak {
        record.best_streak = record.current_streak;
    }
    record.last_productive_date = Some(today_str.clone());

    let mut history: Vec<String> = serde_json::from_str(&record.history_json).unwrap_or_default();
    if !history.contains(&today_str) {
        history.push(today_str);
    }
    record.history_json = serde_json::to_string(&history).unwrap_or_else(|_| "[]".to_string());

    conn.execute(
        "UPDATE streak_records
         SET current_streak = ?1, best_streak = ?2, streak_freezes_available = ?3,
             last_productive_date = ?4, history_json = ?5, updated_at = datetime('now')
         WHERE id = ?6",
        params![
            record.current_streak,
            record.best_streak,
            record.streak_freezes_available,
            record.last_productive_date,
            record.history_json,
            record.id
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(record)
}
