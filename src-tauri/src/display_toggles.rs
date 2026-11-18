use serde::Serialize;

#[derive(Default, Serialize)]
pub struct DisplayToggles {
    hdr: Option<bool>,
    hdr_supported: bool,
    night_light: Option<bool>,
    brightness: Option<u8>,
    error: Option<String>,
}

fn read() -> DisplayToggles {
    DisplayToggles::default()
}

fn write_hdr(_on: bool) -> Result<(), String> {
    Err("HDR control is not supported on this device.".into())
}

fn write_night_light(_on: bool) -> Result<(), String> {
    Err("Night light control is not supported on this device.".into())
}

fn write_brightness(_percent: u8) -> Result<(), String> {
    Err("Brightness control is not supported on this device.".into())
}

async fn run(
    change: impl FnOnce() -> Result<(), String> + Send + 'static,
) -> Result<DisplayToggles, String> {
    tauri::async_runtime::spawn_blocking(move || {
        change()?;
        Ok(read())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_display_toggles() -> Result<DisplayToggles, String> {
    run(|| Ok(())).await
}

#[tauri::command]
pub async fn set_hdr(on: bool) -> Result<DisplayToggles, String> {
    crate::config_gate::require()?;
    run(move || write_hdr(on)).await
}

#[tauri::command]
pub async fn set_night_light(on: bool) -> Result<DisplayToggles, String> {
    crate::config_gate::require()?;
    run(move || write_night_light(on)).await
}

#[tauri::command]
pub async fn set_brightness(percent: u8) -> Result<DisplayToggles, String> {
    crate::config_gate::require()?;
    run(move || write_brightness(percent.min(100))).await
}
