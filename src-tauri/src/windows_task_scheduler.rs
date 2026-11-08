use std::process::Command;

use crate::models::ReminderConfig;

const DAY_NAMES: [&str; 7] = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];

/// Map a `days_of_week_json` array (0=Sun..6=Sat) to schtasks `/D` values.
/// Returns None when the array is empty/unparseable (schtasks then defaults to MON,
/// so callers that need "every day" must pass WEEKLY with `/D *` instead).
pub fn schtasks_days(days_of_week_json: &str) -> Option<String> {
    let days: Vec<i32> = serde_json::from_str(days_of_week_json).ok()?;
    let mut names: Vec<&str> = days
        .into_iter()
        .filter_map(|d| DAY_NAMES.get(d as usize).copied())
        .collect();
    if names.is_empty() {
        return None;
    }
    names.sort();
    names.dedup();
    Some(names.join(","))
}

/// Build the full `schtasks /Create` argument list for a reminder.
/// Pure function (no process spawn) so schedule mapping is unit-testable.
pub fn build_schtasks_args(app_exe_path: &str, reminder: &ReminderConfig) -> Vec<String> {
    let task_full_name = format!("StoryQuestStudio\\{}", reminder.windows_task_name);
    let tr_command = format!(
        "\"{}\" --reminder-mode --reminder-id {}",
        app_exe_path, reminder.id
    );
    let mut args = vec![
        "/Create".to_string(),
        "/F".to_string(),
        "/SC".to_string(),
        "DAILY".to_string(),
        "/TN".to_string(),
        task_full_name,
        "/TR".to_string(),
        tr_command,
        "/ST".to_string(),
        reminder.schedule_time.clone(),
    ];
    match reminder.reminder_type.as_str() {
        "weekly" | "weekly_review" => {
            args[3] = "WEEKLY".to_string();
            match schtasks_days(&reminder.days_of_week_json) {
                Some(days) => {
                    args.push("/D".to_string());
                    args.push(days);
                }
                None => {
                    args.push("/D".to_string());
                    args.push("*".to_string());
                }
            }
        }
        "specific_date" | "task_deadline" => {
            args[3] = "ONCE".to_string();
            if let Some(date) = reminder.specific_date.as_deref().filter(|d| !d.trim().is_empty()) {
                args.push("/SD".to_string());
                args.push(date.to_string());
            }
        }
        _ => {
            args[3] = "DAILY".to_string();
        }
    }
    args
}

/// Registers or updates a per-user Windows Scheduled Task (`schtasks.exe`) so that
/// reminders configured to run even when StoryQuest Studio is completely closed
/// will launch the executable in `--reminder-mode --reminder-id <id>`.
pub fn sync_windows_scheduled_task(
    app_exe_path: &str,
    reminder: &ReminderConfig,
) -> Result<String, String> {
    if !reminder.enabled {
        return remove_windows_scheduled_task(&reminder.windows_task_name);
    }

    let args = build_schtasks_args(app_exe_path, reminder);
    let task_full_name = format!("StoryQuestStudio\\{}", reminder.windows_task_name);

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("schtasks.exe")
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to invoke schtasks.exe: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("schtasks.exe returned error: {}", stderr));
        }
    }

    Ok(format!(
        "Synchronized Windows Task Scheduler entry '{}' at {}",
        task_full_name, reminder.schedule_time
    ))
}

pub fn remove_windows_scheduled_task(windows_task_name: &str) -> Result<String, String> {
    let task_full_name = format!("StoryQuestStudio\\{}", windows_task_name);
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("schtasks.exe")
            .args(["/Delete", "/F", "/TN", &task_full_name])
            .output();
    }
    Ok(format!("Removed scheduled task '{}'", task_full_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reminder(reminder_type: &str, days: &str, specific_date: Option<&str>) -> ReminderConfig {
        ReminderConfig {
            id: "rem_1".to_string(),
            title: "Daily quest".to_string(),
            reminder_type: reminder_type.to_string(),
            task_id: None,
            schedule_time: "09:00".to_string(),
            days_of_week_json: days.to_string(),
            specific_date: specific_date.map(|s| s.to_string()),
            enabled: true,
            snoozed_until: None,
            windows_task_name: "DailyQuest".to_string(),
            last_triggered_at: None,
        }
    }

    fn arg_value(args: &[String], flag: &str) -> Option<String> {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1).cloned())
    }

    #[test]
    fn weekly_includes_days_flag() {
        let args = build_schtasks_args("C:\\app.exe", &reminder("weekly", "[1,3,5]", None));
        assert_eq!(arg_value(&args, "/SC").as_deref(), Some("WEEKLY"));
        assert_eq!(arg_value(&args, "/D").as_deref(), Some("FRI,MON,WED"));
        assert!(arg_value(&args, "/SD").is_none());
    }

    #[test]
    fn weekly_empty_days_means_every_day() {
        let args = build_schtasks_args("C:\\app.exe", &reminder("weekly", "[]", None));
        assert_eq!(arg_value(&args, "/D").as_deref(), Some("*"));
    }

    #[test]
    fn specific_date_includes_start_date() {
        let args = build_schtasks_args(
            "C:\\app.exe",
            &reminder("specific_date", "[1]", Some("2026-12-24")),
        );
        assert_eq!(arg_value(&args, "/SC").as_deref(), Some("ONCE"));
        assert_eq!(arg_value(&args, "/SD").as_deref(), Some("2026-12-24"));
        assert_eq!(arg_value(&args, "/ST").as_deref(), Some("09:00"));
    }

    #[test]
    fn daily_has_no_day_or_date_flags() {
        let args = build_schtasks_args("C:\\app.exe", &reminder("daily", "[1,2]", None));
        assert_eq!(arg_value(&args, "/SC").as_deref(), Some("DAILY"));
        assert!(arg_value(&args, "/D").is_none());
        assert!(arg_value(&args, "/SD").is_none());
    }

    #[test]
    fn task_name_and_command_are_wired() {
        let args = build_schtasks_args("C:\\app.exe", &reminder("daily", "[]", None));
        assert_eq!(
            arg_value(&args, "/TN").as_deref(),
            Some("StoryQuestStudio\\DailyQuest")
        );
        let tr = arg_value(&args, "/TR").unwrap();
        assert!(tr.contains("--reminder-mode --reminder-id rem_1"));
    }
}
