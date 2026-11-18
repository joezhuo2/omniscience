use crate::telemetry::types::StartupApp;

fn list() -> Vec<StartupApp> {
    Vec::new()
}

fn set_enabled(_id: &str, _enabled: bool) -> Result<(), String> {
    Err("Startup apps are not supported on this device.".into())
}

#[tauri::command]
pub async fn get_startup_apps() -> Result<Vec<StartupApp>, String> {
    tauri::async_runtime::spawn_blocking(list)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_startup_app_enabled(id: String, enabled: bool) -> Result<Vec<StartupApp>, String> {
    crate::config_gate::require()?;
    tauri::async_runtime::spawn_blocking(move || {
        set_enabled(&id, enabled)?;
        Ok(list())
    })
    .await
    .map_err(|e| e.to_string())?
}
