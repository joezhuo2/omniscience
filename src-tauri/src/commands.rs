//! Commands the frontend invokes.

use std::sync::Arc;

use tauri::{State, WebviewWindow};

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
