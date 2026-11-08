use chrono::{Local, NaiveTime};
use rusqlite::{params, Connection, OptionalExtension};

pub fn quiet_hours_settings(conn: &Connection) -> (bool, String, String) {
    let get = |key: &str, default: &str| -> String {
        conn.query_row("SELECT value FROM settings WHERE key = ?1", params![key], |row| {
            row.get::<_, String>(0)
        })
        .optional()
        .ok()
        .flatten()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| default.to_string())
    };
    let enabled = get("quiet_hours_enabled", "true") == "true";
    let start = get("quiet_hours_start", "22:30");
    let end = get("quiet_hours_end", "07:30");
    (enabled, start, end)
}

pub fn is_suppressed_by_quiet_hours(conn: &Connection, now_time: NaiveTime) -> bool {
    let (enabled, start, end) = quiet_hours_settings(conn);
    enabled && is_within_quiet_hours(now_time, &start, &end)
}

pub fn is_snoozed(snoozed_until: Option<&str>, now: chrono::DateTime<Local>) -> bool {
    match snoozed_until {
        None => false,
        Some(s) => {
            let s = s.trim();
            if s.is_empty() {
                return false;
            }
            // Stored as "YYYY-MM-DD HH:MM:SS" (local) or RFC3339.
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                return now < dt.with_timezone(&Local);
            }
            if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
                return now.naive_local() < ndt;
            }
            false
        }
    }
}

pub fn log_notification_history(
    conn: &Connection,
    reminder_id: Option<&str>,
    title: &str,
    body: &str,
    delivery_status: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO notification_history (id, reminder_id, title, body, delivery_status)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            format!("notif_{}", uuid::Uuid::new_v4()),
            reminder_id,
            title,
            body,
            delivery_status
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub const APP_AUMID: &str = "com.storyquest.studio";

/// Checks whether current local time falls inside user-configured quiet hours.
pub fn is_within_quiet_hours(now_time: NaiveTime, start_str: &str, end_str: &str) -> bool {
    let start = NaiveTime::parse_from_str(start_str, "%H:%M")
        .unwrap_or_else(|_| NaiveTime::from_hms_opt(22, 30, 0).unwrap());
    let end = NaiveTime::parse_from_str(end_str, "%H:%M")
        .unwrap_or_else(|_| NaiveTime::from_hms_opt(7, 30, 0).unwrap());

    if start <= end {
        now_time >= start && now_time < end
    } else {
        // Crosses midnight (e.g., 22:30 to 07:30)
        now_time >= start || now_time < end
    }
}

/// Dispatches a native Windows Toast Notification containing quest details and actions.
pub fn dispatch_windows_toast(
    quest_title: &str,
    short_description: &str,
    estimated_mins: i32,
    xp_reward: i32,
) -> Result<(), String> {
    let body_line = format!(
        "{} • Est. {} min • +{} XP | Actions: [Start Quest] [Snooze 10m] [Snooze 30m] [Skip Today]",
        short_description, estimated_mins, xp_reward
    );

    #[cfg(target_os = "windows")]
    {
        use winrt_notification::{Duration, Sound, Toast};
        let result = Toast::new(APP_AUMID)
            .title(&format!("StoryQuest Studio — {}", quest_title))
            .text1(&body_line)
            .sound(Some(Sound::Default))
            .duration(Duration::Long)
            .show();
        if let Err(e) = result {
            return Err(format!("Toast notification permission or delivery failure: {}", e));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        log::info!("Toast Notification [{}]: {} - {}", APP_AUMID, quest_title, body_line);
    }

    Ok(())
}

/// Collects any missed reminders while the computer was asleep or powered off and
/// consolidates them into a single non-guilt recovery summary rather than flooding notifications.
pub fn recover_missed_reminders_summary(conn: &Connection, current_hhmm: &str) -> Result<Vec<String>, String> {
    let today_date = Local::now().date_naive().to_string();
    let mut stmt = conn
        .prepare(
            "SELECT id, title, schedule_time FROM reminders
             WHERE enabled = 1 AND schedule_time < ?1
             AND (last_triggered_at IS NULL OR substr(last_triggered_at, 1, 10) != ?2)",
        )
        .map_err(|e| e.to_string())?;

    let missed: Vec<(String, String, String)> = stmt
        .query_map(params![current_hhmm, &today_date], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .collect();

    let mut summaries = Vec::new();
    for (rem_id, title, sched_time) in &missed {
        summaries.push(format!("{} (scheduled for {})", title, sched_time));
        let _ = conn.execute(
            "UPDATE reminders SET last_triggered_at = datetime('now') WHERE id = ?1",
            params![rem_id],
        );
    }

    if !summaries.is_empty() {
        let combined_body = format!(
            "Welcome back! You have {} scheduled quest reminder(s) ready whenever you want to begin: {}",
            summaries.len(),
            summaries.join(", ")
        );
        let _ = conn.execute(
            "INSERT INTO notification_history (id, title, body, delivery_status, user_action)
             VALUES (?1, ?2, ?3, 'missed_recovered', 'summary_shown')",
            params![
                format!("notif_{}", uuid::Uuid::new_v4()),
                "StoryQuest Ready When You Are",
                combined_body
            ],
        );
    }

    Ok(summaries)
}
