//! Tray icon: the app's only presence (no taskbar button, no Alt+Tab entry).

use std::sync::Arc;

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

use crate::shell::{OPEN_SETTINGS_EVENT, Shell, TrayChecks};

const SHOW_HIDE: &str = "show_hide";
const LAUNCH_AT_STARTUP: &str = "launch_at_startup";
const AUTO_CHECK_UPDATES: &str = "auto_check_updates";
const HIDE_OVER_FULLSCREEN: &str = "hide_over_fullscreen";
const CHECK_UPDATES: &str = "check_updates";
const SETTINGS: &str = "settings";
const QUIT: &str = "quit";

/// Emitted when the user picks "Check for updates now".
pub const CHECK_UPDATE_EVENT: &str = "omniscience://check-update";

pub fn init(app: &AppHandle) -> tauri::Result<()> {
    let checks = TrayChecks {
        launch_at_startup: CheckMenuItem::with_id(
            app,
            LAUNCH_AT_STARTUP,
            "Launch at startup",
            true,
            false,
            None::<&str>,
        )?,
        auto_check_updates: CheckMenuItem::with_id(
            app,
            AUTO_CHECK_UPDATES,
            "Auto-check for updates",
            true,
            false,
            None::<&str>,
        )?,
        hide_over_fullscreen: CheckMenuItem::with_id(
            app,
            HIDE_OVER_FULLSCREEN,
            "Hide over fullscreen apps",
            true,
            false,
            None::<&str>,
        )?,
    };
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(
                app,
                SHOW_HIDE,
                "Show / Hide Omniscience",
                true,
                None::<&str>,
            )?,
            &PredefinedMenuItem::separator(app)?,
            &checks.launch_at_startup,
            &checks.auto_check_updates,
            &checks.hide_over_fullscreen,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(
                app,
                CHECK_UPDATES,
                "Check for updates now",
                true,
                None::<&str>,
            )?,
            &MenuItem::with_id(app, SETTINGS, "Settings…", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, QUIT, "Quit", true, None::<&str>)?,
        ],
    )?;
    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("Omniscience")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| on_menu(app, event.id().as_ref()))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                app.state::<Arc<Shell>>().toggle(app);
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    app.state::<Arc<Shell>>().set_tray(checks);
    Ok(())
}

fn on_menu(app: &AppHandle, id: &str) {
    let shell = app.state::<Arc<Shell>>();
    let mut s = shell.settings();
    match id {
        SHOW_HIDE => return shell.toggle(app),
        QUIT => return app.exit(0),
        CHECK_UPDATES => {
            shell.show(app);
            let _ = app.emit(CHECK_UPDATE_EVENT, ());
            return;
        }
        SETTINGS => {
            shell.show(app);
            // Focus so the settings fields accept typing.
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.set_focus();
            }
            let _ = app.emit(OPEN_SETTINGS_EVENT, ());
            return;
        }
        LAUNCH_AT_STARTUP => s.launch_at_startup = !s.launch_at_startup,
        AUTO_CHECK_UPDATES => s.auto_check_updates = !s.auto_check_updates,
        HIDE_OVER_FULLSCREEN => s.hide_over_fullscreen = !s.hide_over_fullscreen,
        _ => return,
    }
    if let Err(e) = shell.update_settings(app, s) {
        eprintln!("tray setting: {e}");
        // The menu already toggled its checkmark; put it back to the unchanged setting.
        shell.sync_tray();
    }
}
