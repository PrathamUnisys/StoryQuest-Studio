pub mod ai_planner;
pub mod commands;
pub mod credentials;
pub mod db;
pub mod gamification;
pub mod mcp_integration;
pub mod models;
pub mod project_fs;
pub mod scheduler;
pub mod tray;
pub mod windows_task_scheduler;
pub mod youtube;
#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::sync::Mutex;

use commands::*;
use tauri::{Manager, WindowEvent};

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .cloned()
}

fn data_dir_from_args(args: &[String]) -> Option<PathBuf> {
    arg_value(args, "--data-dir")
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
}

fn data_dir_from_env() -> Option<PathBuf> {
    std::env::var("STORYQUEST_DATA_DIR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
}

fn migrate_only(args: &[String]) -> Result<(), String> {
    let data_dir = data_dir_from_args(args)
        .or_else(data_dir_from_env)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("StoryQuestProjects")
        });
    let db_path = data_dir.join("storyquest.db");
    let conn = db::init_database(&db_path)?;
    println!(
        "StoryQuest database ready: {} (schema {})",
        db_path.display(),
        db::schema_version(&conn)?
    );
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = dotenvy::dotenv();
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|arg| arg == "--migrate-only") {
        if let Err(error) = migrate_only(&args) {
            eprintln!("Migration failed: {error}");
            std::process::exit(1);
        }
        return;
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
            if argv.iter().any(|arg| arg == "--reminder-mode") {
                let _ = scheduler::dispatch_windows_toast(
                    "Scheduled StoryQuest Reminder",
                    "Your daily animation quest is ready to start.",
                    25,
                    25,
                );
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(move |app| {
            let data_dir = data_dir_from_args(&args)
                .or_else(data_dir_from_env)
                .or_else(|| app.path().app_local_data_dir().ok())
                .unwrap_or_else(|| std::env::current_dir().unwrap());
            let db_path = data_dir.join("storyquest.db");
            let conn = db::init_database(&db_path).expect("Failed to initialize SQLite database");

            let now_hhmm = chrono::Local::now().format("%H:%M").to_string();
            let _ = scheduler::recover_missed_reminders_summary(&conn, &now_hhmm);

            let start_minimized = args.iter().any(|arg| arg == "--minimized")
                || conn
                    .query_row(
                        "SELECT value FROM settings WHERE key = 'start_minimized'",
                        [],
                        |row| row.get::<_, String>(0),
                    )
                    .map(|value| value == "true")
                    .unwrap_or(false);

            app.manage(AppState {
                db: Mutex::new(conn),
                app_dir: data_dir,
                db_path,
            });

            tray::setup_system_tray(app.handle())?;

            if start_minimized {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let minimize_to_tray = window
                    .app_handle()
                    .state::<AppState>()
                    .db
                    .lock()
                    .ok()
                    .and_then(|conn| {
                        conn.query_row(
                            "SELECT value FROM settings WHERE key = 'minimize_to_tray'",
                            [],
                            |row| row.get::<_, String>(0),
                        )
                        .ok()
                    })
                    .map(|value| value != "false")
                    .unwrap_or(true);

                if minimize_to_tray {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            cmd_get_bootstrap,
            cmd_update_profile,
            cmd_complete_onboarding,
            cmd_get_settings,
            cmd_set_setting,
            cmd_get_creator_ranks,
            cmd_update_creator_thresholds,
            cmd_recalculate_creator_rank,
            cmd_get_app_info,
            cmd_open_data_folder,
            cmd_list_campaigns,
            cmd_list_episodes,
            cmd_list_episode_tasks,
            cmd_list_reminders,
            cmd_apply_autostart,
            cmd_award_quest_xp,
            cmd_unlock_dependent_tasks,
            cmd_record_streak,
            cmd_generate_ai_roadmap,
            cmd_scaffold_campaign_folders,
            cmd_open_project_folder,
            cmd_save_api_key,
            cmd_has_api_key,
            cmd_remove_api_key,
            cmd_create_campaign,
            cmd_update_campaign,
            cmd_archive_campaign,
            cmd_create_episode,
            cmd_update_episode,
            cmd_set_task_status,
            cmd_reopen_task,
            cmd_get_achievement_notifications,
            cmd_acknowledge_achievement_notification,
            cmd_set_streak_prefs,
            cmd_trigger_test_toast,
            cmd_backup_database,
            cmd_youtube_set_client_credentials,
            cmd_youtube_has_client_credentials,
            cmd_youtube_connect,
            cmd_youtube_sync_now,
            cmd_youtube_disconnect,
        ])
        .run(tauri::generate_context!())
        .expect("error while running StoryQuest Studio Tauri application");
}
