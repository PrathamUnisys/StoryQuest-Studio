use std::process::Command;
use crate::models::ReminderConfig;

/// Registers or updates a per-user Windows Scheduled Task (`schtasks.exe`) so that
/// reminders configured to run even when StoryQuest Studio is completely closed
/// will launch the executable in `--reminder-mode --reminder-id <id>`.
pub fn sync_windows_scheduled_task(
    app_exe_path: &str,
    reminder: &ReminderConfig,
) -> Result<String, String> {
    let task_full_name = format!("StoryQuestStudio\\{}", reminder.windows_task_name);

    if !reminder.enabled {
        return remove_windows_scheduled_task(&reminder.windows_task_name);
    }

    let tr_command = format!(
        "\"{}\" --reminder-mode --reminder-id {}",
        app_exe_path, reminder.id
    );
    let schedule_flag = match reminder.reminder_type.as_str() {
        "weekly" | "weekly_review" => "WEEKLY",
        "specific_date" | "task_deadline" => "ONCE",
        _ => "DAILY",
    };

    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("schtasks.exe");
        cmd.args([
            "/Create",
            "/F", // Force overwrite to prevent duplicate scheduled tasks
            "/SC",
            schedule_flag,
            "/TN",
            &task_full_name,
            "/TR",
            &tr_command,
            "/ST",
            &reminder.schedule_time,
        ]);
        let output = cmd
            .output()
            .map_err(|e| format!("Failed to invoke schtasks.exe: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("schtasks.exe returned error: {}", stderr));
        }
    }

    Ok(format!(
        "Synchronized Windows Task Scheduler entry '{}' at {} ({})",
        task_full_name, reminder.schedule_time, schedule_flag
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
