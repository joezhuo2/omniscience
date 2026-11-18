use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::plugin::{Builder, TauriPlugin};
use tauri::{AppHandle, Manager, State, Wry};

use crate::settings::{self, Settings};
use crate::telemetry::{Control, Telemetry};

struct Embedded {
    dir: Option<PathBuf>,
    settings: Mutex<Settings>,
    started: AtomicBool,
}

macro_rules! commands {
    ($($module:ident :: $name:ident),* $(,)?) => {
        pub const COMMANDS: &[&str] = &[$(stringify!($name)),*];

        pub fn plugin() -> TauriPlugin<Wry> {
            Builder::new("omniscience")
                .invoke_handler(tauri::generate_handler![$(crate::$module::$name),*])
                .setup(|app, _api| {
                    setup(app);
                    Ok(())
                })
                .build()
        }
    };
}

commands![
    embed::set_expanded,
    commands::kill_process_group,
    commands::process_details,
    commands::open_process_location,
    commands::set_process_priority,
    commands::set_process_efficiency,
    commands::suspend_processes,
    commands::resume_processes,
    commands::kill_process_tree,
    embed::get_settings,
    embed::set_settings,
    settings::get_default_settings,
    commands::get_system_summary,
    diagnostics::get_diagnostic_report,
    commands::get_telemetry_history,
    commands::get_history_events,
    commands::export_history,
    embed::set_active,
    commands::set_process_sampling,
];

fn setup(app: &AppHandle) {
    let dir = app
        .path()
        .app_data_dir()
        .ok()
        .map(|d| d.join("omniscience"));
    let settings = dir.as_deref().map(settings::load).unwrap_or_default();
    let telemetry = Telemetry::new(Control {
        visible: false,
        ..Control::default()
    });
    telemetry.update(|c| settings.apply(c));
    telemetry.open_history(dir.as_deref());
    app.manage(telemetry);
    app.manage(Embedded {
        dir,
        settings: Mutex::new(settings),
        started: AtomicBool::new(false),
    });
}

#[tauri::command]
fn set_expanded(expanded: bool, telemetry: State<'_, Arc<Telemetry>>) {
    telemetry.update(|c| c.expanded = expanded);
}

#[tauri::command]
fn set_active(
    active: bool,
    app: AppHandle,
    telemetry: State<'_, Arc<Telemetry>>,
    embedded: State<'_, Embedded>,
) {
    if active && !embedded.started.swap(true, Ordering::SeqCst) {
        telemetry.start(app);
    }
    telemetry.update(|c| c.visible = active);
}

#[tauri::command]
fn get_settings(embedded: State<'_, Embedded>) -> Settings {
    embedded
        .settings
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

#[tauri::command]
fn set_settings(
    settings: Settings,
    telemetry: State<'_, Arc<Telemetry>>,
    embedded: State<'_, Embedded>,
) -> Result<Settings, String> {
    let new = settings.sanitized();
    telemetry.update(|c| new.apply(c));
    if let Some(dir) = &embedded.dir {
        settings::save(dir, &new)?;
    }
    *embedded.settings.lock().unwrap_or_else(|e| e.into_inner()) = new.clone();
    Ok(new)
}

#[cfg(test)]
mod tests {
    #[test]
    fn lists_every_plugin_command() {
        assert_eq!(
            super::COMMANDS,
            [
                "set_expanded",
                "kill_process_group",
                "process_details",
                "open_process_location",
                "set_process_priority",
                "set_process_efficiency",
                "suspend_processes",
                "resume_processes",
                "kill_process_tree",
                "get_settings",
                "set_settings",
                "get_default_settings",
                "get_system_summary",
                "get_diagnostic_report",
                "get_telemetry_history",
                "get_history_events",
                "export_history",
                "set_active",
                "set_process_sampling",
            ]
        );
    }
}
