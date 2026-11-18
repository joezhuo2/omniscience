//! Commands the frontend invokes.

use std::sync::Arc;

use tauri::State;
#[cfg(feature = "standalone")]
use tauri::{AppHandle, WebviewWindow};

#[cfg(feature = "standalone")]
use crate::positions::Positions;
#[cfg(feature = "standalone")]
use crate::settings::Settings;
#[cfg(feature = "standalone")]
use crate::shell::Shell;
use crate::telemetry::history::HistorySample;
use crate::telemetry::types::KillReport;
use crate::telemetry::{Telemetry, processes};
#[cfg(feature = "standalone")]
use crate::{webview, window};

#[cfg(feature = "standalone")]
/// Opens or closes the dashboard: snaps the native window to the expanded bounds before the
/// expand animation (or back to the pill after the collapse animation), switches detail-tier
/// sampling, and sets the WebView2 memory level.
#[tauri::command]
pub fn set_expanded(
    expanded: bool,
    window: WebviewWindow,
    telemetry: State<'_, Arc<Telemetry>>,
    positions: State<'_, Arc<Positions>>,
) -> Result<(), String> {
    telemetry.update(|c| c.expanded = expanded);
    let height = if expanded {
        window::EXPANDED_MAX_HEIGHT
    } else {
        window::COLLAPSED_SIZE.1
    };
    window::set_height(&window, height).map_err(|e| e.to_string())?;
    positions
        .keep_on_screen(&window, height)
        .map_err(|e| e.to_string())?;
    // Collapsed, clicks pass through the transparent margins around the pill.
    window
        .set_ignore_cursor_events(!expanded)
        .map_err(|e| e.to_string())?;
    webview::set_memory_low(&window, !expanded);
    Ok(())
}

#[cfg(feature = "standalone")]
/// Fits the expanded window to the dashboard content height (logical px).
#[tauri::command]
pub fn set_overlay_height(
    height: f64,
    window: WebviewWindow,
    positions: State<'_, Arc<Positions>>,
) -> Result<(), String> {
    window::set_height(&window, height).map_err(|e| e.to_string())?;
    positions
        .keep_on_screen(&window, window::clamp_height(height))
        .map_err(|e| e.to_string())
}

/// Turns process-list sampling on while the processes section is open and visible.
#[tauri::command]
pub fn set_process_sampling(
    enabled: bool,
    window: tauri::Window,
    telemetry: State<'_, Arc<Telemetry>>,
) {
    telemetry.set_process_viewer(window.label(), enabled);
}

#[cfg(feature = "standalone")]
const WIDGET_BROWSER_ARGS: &str =
    "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --disable-gpu";

#[cfg(feature = "standalone")]
#[tauri::command]
pub async fn open_widget(app: AppHandle) -> Result<(), String> {
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

    if let Some(w) = app.get_webview_window("widget") {
        let _ = w.unminimize();
        return w.set_focus().map_err(|e| e.to_string());
    }
    WebviewWindowBuilder::new(
        &app,
        "widget",
        WebviewUrl::App("index.html?widget=1".into()),
    )
    .title("Omniscience")
    .inner_size(420.0, 640.0)
    .min_inner_size(320.0, 240.0)
    .resizable(true)
    .always_on_top(true)
    .decorations(true)
    .skip_taskbar(false)
    .additional_browser_args(WIDGET_BROWSER_ARGS)
    .build()
    .map(|_| ())
    .map_err(|e| e.to_string())
}

#[cfg(feature = "standalone")]
#[tauri::command]
pub fn set_widget_active(active: bool, telemetry: State<'_, Arc<Telemetry>>) {
    telemetry.update(|c| c.widget = active);
}

/// Ends every process in the group `key`, refusing protected ones.
#[tauri::command]
pub async fn kill_process_group(key: String) -> KillReport {
    tauri::async_runtime::spawn_blocking(move || processes::kill_group(&key))
        .await
        .unwrap_or_else(|e| KillReport {
            key: String::new(),
            results: vec![crate::telemetry::types::KillResult {
                pid: 0,
                outcome: crate::telemetry::types::KillOutcome::Failed,
                message: Some(e.to_string()),
            }],
        })
}

#[cfg(feature = "standalone")]
#[tauri::command]
pub fn get_settings(shell: State<'_, Arc<Shell>>) -> Settings {
    shell.settings()
}

#[cfg(feature = "standalone")]
/// Applies and saves settings; returns the values actually applied (after clamping).
#[tauri::command]
pub fn set_settings(
    settings: Settings,
    app: AppHandle,
    shell: State<'_, Arc<Shell>>,
) -> Result<Settings, String> {
    shell.update_settings(&app, settings)
}

#[cfg(feature = "standalone")]
/// Moves the overlay back to the top-center of the primary monitor and forgets the saved spot.
#[tauri::command]
pub fn reset_position(
    window: WebviewWindow,
    positions: State<'_, Arc<Positions>>,
) -> Result<(), String> {
    positions.set_tracking(false);
    let result = window::place_top_center(&window).map_err(|e| e.to_string());
    positions.forget(&window);
    positions.set_tracking(true);
    result
}

#[tauri::command]
pub fn get_telemetry_history(
    since_ms: Option<u64>,
    telemetry: State<'_, Arc<Telemetry>>,
) -> Vec<HistorySample> {
    telemetry.history(since_ms)
}
