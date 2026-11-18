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

#[cfg(windows)]
const THROTTLE_THERMAL_POLICY: u32 = 0x0012_0075;

#[cfg(windows)]
fn mode_from_value(value: u32) -> Option<FanMode> {
    match value {
        0 => Some(FanMode::Performance),
        1 => Some(FanMode::Turbo),
        2 => Some(FanMode::Silent),
        _ => None,
    }
}

#[cfg(windows)]
fn value_from_mode(mode: FanMode) -> u32 {
    match mode {
        FanMode::Performance => 0,
        FanMode::Turbo => 1,
        FanMode::Silent => 2,
    }
}

#[cfg(windows)]
pub fn read() -> FanModeStatus {
    let value = crate::asus::Asus::open().and_then(|asus| asus.get(THROTTLE_THERMAL_POLICY));
    FanModeStatus {
        mode: value.as_ref().ok().and_then(|&v| mode_from_value(v)),
        supported: value.is_ok(),
        error: None,
    }
}
#[cfg(not(windows))]
pub fn read() -> FanModeStatus {
    FanModeStatus {
        mode: None,
        supported: false,
        error: None,
    }
}

#[cfg(windows)]
pub fn write(mode: FanMode) -> Result<(), String> {
    crate::asus::Asus::open()?.set(THROTTLE_THERMAL_POLICY, value_from_mode(mode))
}
#[cfg(not(windows))]
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

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn maps_firmware_values_both_ways() {
        for mode in [FanMode::Silent, FanMode::Performance, FanMode::Turbo] {
            assert_eq!(mode_from_value(value_from_mode(mode)), Some(mode));
        }
        assert_eq!(mode_from_value(0), Some(FanMode::Performance));
        assert_eq!(mode_from_value(1), Some(FanMode::Turbo));
        assert_eq!(mode_from_value(2), Some(FanMode::Silent));
        assert_eq!(mode_from_value(3), None);
    }

    #[test]
    #[ignore]
    fn live_read_only() {
        let status = read();
        println!(
            "fan mode: {:?} supported: {} error: {:?}",
            status.mode, status.supported, status.error
        );
    }
}
