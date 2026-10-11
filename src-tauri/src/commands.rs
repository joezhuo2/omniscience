//! Commands the frontend invokes.

use std::sync::Arc;

use tauri::State;

use crate::telemetry::types::KillReport;
use crate::telemetry::{Telemetry, processes};

/// Starts or stops detail-tier sampling (dashboard open or closed).
#[tauri::command]
pub fn set_expanded(expanded: bool, telemetry: State<'_, Arc<Telemetry>>) {
    telemetry.update(|c| c.expanded = expanded);
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
