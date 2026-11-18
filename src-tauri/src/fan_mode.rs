use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FanMode {
    Silent,
    Performance,
    Turbo,
}

#[derive(Serialize)]
pub struct FanModeStatus {
    mode: Option<FanMode>,
    supported: bool,
    error: Option<String>,
}

pub fn read() -> FanModeStatus {
    FanModeStatus {
        mode: None,
        supported: false,
        error: None,
    }
}

pub fn write(_mode: FanMode) -> Result<(), String> {
    Err("Fan modes are not supported on this device.".into())
}

#[tauri::command]
pub async fn get_fan_mode() -> Result<FanModeStatus, String> {
    tauri::async_runtime::spawn_blocking(read)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_fan_mode(mode: FanMode) -> Result<FanModeStatus, String> {
    crate::config_gate::require()?;
    tauri::async_runtime::spawn_blocking(move || {
        write(mode)?;
        Ok(read())
    })
    .await
    .map_err(|e| e.to_string())?
}
