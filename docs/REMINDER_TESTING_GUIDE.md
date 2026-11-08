# StoryQuest Studio — Windows Startup, Tray & Reminder Testing Guide

## 1. Architecture of the Dual-Layer Reminder Engine
StoryQuest Studio never relies on fragile browser `setTimeout` timers alone:
1. **Layer 1 — Rust Background Scheduler (`src-tauri/src/scheduler.rs`)**: Runs inside the native Rust process while the application is open or minimized to the Windows System Tray.
2. **Layer 2 — Windows Task Scheduler Integration (`src-tauri/src/windows_task_scheduler.rs`)**: Registers per-user Windows Scheduled Tasks under `StoryQuestStudio\Reminder_<id>` using `schtasks.exe /Create /F`. When StoryQuest Studio is completely closed, Windows launches `storyquest-studio.exe --reminder-mode --reminder-id <id>` at the scheduled time to fire the native Windows Toast notification.

## 2. Step-by-Step Verification Checklist
1. **Test Native Windows Toast Notification**:
   - Navigate to **Calendar & Reminders** and click **Test Windows Toast**.
   - Confirm the toast displays the app identity (`StoryQuest Studio`), quest title, duration, XP reward, and action buttons (`Start Quest`, `Snooze 10m`, `Snooze 30m`, `Reschedule`, `Skip Today`).
2. **Test Minimize-to-Tray on Window Close**:
   - Click the standard Windows `X` close button on the top-right title bar.
   - Verify the window hides cleanly to the Windows System Tray without terminating the background process.
   - Right-click the tray icon and click **Quit Application** to perform a full exit.
3. **Test Windows Task Scheduler Registration**:
   - Create or toggle a reminder in the UI.
   - Open PowerShell and run:
     ```powershell
     schtasks.exe /Query /TN "StoryQuestStudio\Reminder_rem_01" /V /FO LIST
     ```
   - Verify the task exists, is not duplicated on repeated updates (`/F` flag), and is removed when disabled.
4. **Test Quiet Hours & Missed Reminder Recovery**:
   - Enable Quiet Hours (`22:30` to `07:30`). Verify reminders during that window log `quiet_hours_suppressed` instead of interrupting sleep.
   - Simulate returning from PC sleep or power-off: verify `recover_missed_reminders_summary()` presents a single consolidated welcome summary rather than a flood of old toasts.
