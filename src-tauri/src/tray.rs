//! Tray icon: the app's only presence (no taskbar button, no Alt+Tab entry).

use std::sync::Arc;
use std::time::Duration;

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

use crate::shell::{OPEN_SETTINGS_EVENT, Shell, TrayChecks, now_ms};
use crate::telemetry::Telemetry;
use crate::telemetry::types::CoreMetrics;

const SHOW_HIDE: &str = "show_hide";
const LAUNCH_AT_STARTUP: &str = "launch_at_startup";
const AUTO_CHECK_UPDATES: &str = "auto_check_updates";
const HIDE_OVER_FULLSCREEN: &str = "hide_over_fullscreen";
const CHECK_UPDATES: &str = "check_updates";
const SYSTEM_UPDATES: &str = "system_updates";
const SNOOZE_ALERTS: &str = "snooze_alerts";
const SNOOZE_MS: u64 = 3_600_000;
pub const OPEN_SYSTEM_UPDATES_EVENT: &str = "omniscience://open-system-updates";
const SETTINGS: &str = "settings";
const QUIT: &str = "quit";

/// Emitted when the user picks "Check for updates now".
pub const CHECK_UPDATE_EVENT: &str = "omniscience://check-update";

pub fn init(app: &AppHandle) -> tauri::Result<()> {
    let launch_at_startup = CheckMenuItem::with_id(
        app,
        LAUNCH_AT_STARTUP,
        "Launch at startup",
        true,
        false,
        None::<&str>,
    )?;
    let auto_check_updates = CheckMenuItem::with_id(
        app,
        AUTO_CHECK_UPDATES,
        "Auto-check for updates",
        true,
        false,
        None::<&str>,
    )?;
    let hide_over_fullscreen = CheckMenuItem::with_id(
        app,
        HIDE_OVER_FULLSCREEN,
        "Hide over fullscreen apps",
        true,
        false,
        None::<&str>,
    )?;
    let system_updates = MenuItem::with_id(
        app,
        SYSTEM_UPDATES,
        "Check for system updates",
        true,
        None::<&str>,
    )?;
    let snooze = MenuItem::with_id(
        app,
        SNOOZE_ALERTS,
        "Snooze alerts for 1 hour",
        true,
        None::<&str>,
    )?;
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
            &launch_at_startup,
            &auto_check_updates,
            &hide_over_fullscreen,
            &snooze,
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
    let checks = TrayChecks {
        launch_at_startup,
        auto_check_updates,
        hide_over_fullscreen,
        menu: menu.clone(),
        system_updates,
        snooze,
    };
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
    spawn_tooltip(app.clone());
    Ok(())
}

fn tooltip(core: &CoreMetrics) -> String {
    let mut text = format!("Omniscience\nCPU {:.0}%", core.cpu.percent);
    if let Some(t) = core.cpu_temp_c {
        text.push_str(&format!(" {t:.0}°C"));
    }
    text.push_str(&format!("\nRAM {:.0}%", core.ram.percent));
    if let Some(gpu) = core.dgpu.as_ref().or(core.igpu.as_ref()) {
        text.push_str(&format!("\nGPU {:.0}%", gpu.percent));
    }
    if let Some(disk) = &core.disk {
        text.push_str(&format!("\nDisk {:.0}%", disk.percent));
    }
    text
}

fn spawn_tooltip(app: AppHandle) {
    std::thread::Builder::new()
        .name("tray-tooltip".into())
        .spawn(move || {
            let mut shown = String::new();
            let mut snoozed = false;
            loop {
                std::thread::sleep(Duration::from_secs(2));
                let shell = app.state::<Arc<Shell>>();
                let now_snoozed = shell.settings().alerts.snooze_until_ms > now_ms();
                if now_snoozed != snoozed {
                    snoozed = now_snoozed;
                    shell.sync_tray();
                }
                let Some(snapshot) = app.state::<Arc<Telemetry>>().latest() else {
                    continue;
                };
                let text = tooltip(&snapshot.core);
                if text == shown {
                    continue;
                }
                if let Some(tray) = app.tray_by_id("main")
                    && tray.set_tooltip(Some(&text)).is_ok()
                {
                    shown = text;
                }
            }
        })
        .expect("spawn tray tooltip");
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
        SYSTEM_UPDATES => {
            app.state::<Arc<crate::driver_updates::UpdateChecker>>()
                .start();
            shell.show(app);
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
            let _ = app.emit(OPEN_SYSTEM_UPDATES_EVENT, ());
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
        SNOOZE_ALERTS => {
            let now = now_ms();
            s.alerts.snooze_until_ms = if s.alerts.snooze_until_ms > now {
                0
            } else {
                now + SNOOZE_MS
            };
        }
        _ => return,
    }
    if let Err(e) = shell.update_settings(app, s) {
        crate::diagnostics::log_error(format_args!("tray setting: {e}"));
        // The menu already toggled its checkmark; put it back to the unchanged setting.
        shell.sync_tray();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::types::{Level, MemoryMetric, Metric, NetMetric, Thresholds};

    fn core() -> CoreMetrics {
        let t = Thresholds::default();
        CoreMetrics {
            cpu: Metric::new(23.4, t),
            cpu_mhz: None,
            cpu_temp_c: Some(61.0),
            igpu_temp_c: None,
            dgpu_temp_c: None,
            ram: MemoryMetric {
                used_bytes: 1,
                total_bytes: 2,
                percent: 54.0,
                level: Level::Normal,
            },
            disk: Some(Metric::new(3.0, t)),
            net: NetMetric {
                down_bps: 0.0,
                up_bps: 0.0,
                down_pct: 0.0,
                up_pct: 0.0,
                level: Level::Normal,
            },
            igpu: Some(Metric::new(5.0, t)),
            dgpu: Some(Metric::new(12.0, t)),
            package_watts: None,
        }
    }

    #[test]
    fn tooltip_lists_every_value() {
        assert_eq!(
            tooltip(&core()),
            "Omniscience\nCPU 23% 61°C\nRAM 54%\nGPU 12%\nDisk 3%"
        );
    }

    #[test]
    fn tooltip_omits_missing_values() {
        let mut c = core();
        c.cpu_temp_c = None;
        c.disk = None;
        c.dgpu = None;
        assert_eq!(tooltip(&c), "Omniscience\nCPU 23%\nRAM 54%\nGPU 5%");
        c.igpu = None;
        assert_eq!(tooltip(&c), "Omniscience\nCPU 23%\nRAM 54%");
    }
}
