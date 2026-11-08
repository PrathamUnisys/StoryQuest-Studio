use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime,
};

pub fn setup_system_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let open_i = MenuItem::with_id(app, "open_app", "Open StoryQuest", true, None::<&str>)?;
    let today_i = MenuItem::with_id(app, "todays_quest", "Today's Quest", true, None::<&str>)?;
    let focus_i = MenuItem::with_id(app, "start_focus", "Start Focus Session", true, None::<&str>)?;
    let snooze_i = MenuItem::with_id(app, "snooze_reminders", "Snooze Reminders (30m)", true, None::<&str>)?;
    let pause_i = MenuItem::with_id(app, "pause_reminders", "Pause Reminders", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit_app", "Quit Application", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&open_i, &today_i, &focus_i, &snooze_i, &pause_i, &quit_i],
    )?;

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("StoryQuest Studio — Every episode is a quest.")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open_app" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "todays_quest" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                    let _ = w.emit("tray://navigate", "dashboard");
                }
            }
            "start_focus" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                    let _ = w.emit("tray://start-focus", true);
                }
            }
            "snooze_reminders" => {
                let _ = app.emit("tray://snooze-reminders", 30);
            }
            "pause_reminders" => {
                let _ = app.emit("tray://pause-reminders", true);
            }
            "quit_app" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    let _tray = builder.build(app)?;

    Ok(())
}
