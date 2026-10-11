//! Commands the frontend invokes.

use std::sync::Arc;

use tauri::{AppHandle, State, WebviewWindow};

use crate::positions::Positions;
use crate::settings::Settings;
use crate::shell::Shell;
use crate::telemetry::types::KillReport;
use crate::telemetry::{Telemetry, processes};
use crate::{webview, window};

/// Opens or closes the dashboard: snaps the native window to the expanded bounds before the
/// expand animation (or back to the pill after the collapse animation), switches detail-tier
/// sampling, and sets the WebView2 memory level.
#[tauri::command]
pub fn set_expanded(
    expanded: bool,
    window: WebviewWindow,
    telemetry: State<'_, Arc<Telemetry>>,
) -> Result<(), String> {
    telemetry.update(|c| c.expanded = expanded);
    let height = if expanded {
        window::EXPANDED_MAX_HEIGHT
    } else {
        window::COLLAPSED_SIZE.1
    };
    window::set_height(&window, height).map_err(|e| e.to_string())?;
    // Collapsed, clicks pass through the transparent margins around the pill.
    window
        .set_ignore_cursor_events(!expanded)
        .map_err(|e| e.to_string())?;
    webview::set_memory_low(&window, !expanded);
    Ok(())
}

/// Fits the expanded window to the dashboard content height (logical px).
#[tauri::command]
pub fn set_overlay_height(height: f64, window: WebviewWindow) -> Result<(), String> {
    window::set_height(&window, height).map_err(|e| e.to_string())
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

#[tauri::command]
pub fn get_settings(shell: State<'_, Arc<Shell>>) -> Settings {
    shell.settings()
}

/// Applies and saves settings; returns the values actually applied (after clamping).
#[tauri::command]
pub fn set_settings(
    settings: Settings,
    app: AppHandle,
    shell: State<'_, Arc<Shell>>,
) -> Result<Settings, String> {
    shell.update_settings(&app, settings)
}

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
