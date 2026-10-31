//! GPU configuration and opt-in AC/battery automation.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GpuMode {
    Auto,
    Med,
    Min,
    Max,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
struct Preferences {
    mode: Option<GpuMode>,
    pending_boot: Option<u64>,
    #[serde(skip)]
    error: Option<String>,
}

pub struct GpuConfig {
    path: PathBuf,
    prefs: Mutex<Preferences>,
}

#[derive(Serialize)]
pub struct GpuStatus {
    selected: Option<GpuMode>,
    current: Option<GpuMode>,
    supported: bool,
    mux_supported: bool,
    restart_required: bool,
    error: Option<String>,
}

impl GpuConfig {
    pub fn start(app: &AppHandle) -> Result<Arc<Self>, String> {
        let path = app
            .path()
            .app_config_dir()
            .map_err(|e| e.to_string())?
            .join("gpu-mode.json");
        let mut prefs: Preferences = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        if prefs
            .pending_boot
            .is_some_and(|boot| boot.abs_diff(boot_id()) > 5)
        {
            prefs.pending_boot = None;
        }
        let config = Arc::new(Self {
            path,
            prefs: Mutex::new(prefs),
        });
        #[cfg(windows)]
        {
            let worker = config.clone();
            std::thread::spawn(move || {
                let mut previous = None;
                let mut initial = true;
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    let Ok(mut prefs) = worker.prefs.lock() else {
                        continue;
                    };
                    if prefs.pending_boot.is_some() {
                        continue;
                    }
                    if let Some(mode) = prefs.mode
                        && (initial || mode == GpuMode::Auto)
                    {
                        match ac_power() {
                            Ok(ac) if initial || previous != Some(ac) => {
                                previous = Some(ac);
                                match apply(mode, ac) {
                                    Ok(restart) => {
                                        prefs.pending_boot = restart.then(boot_id);
                                        prefs.error = worker.save(&prefs).err();
                                    }
                                    Err(e) => prefs.error = Some(e),
                                }
                            }
                            Err(e) => prefs.error = Some(e),
                            _ => {}
                        }
                    }
                    initial = false;
                }
            });
        }
        Ok(config)
    }

    fn save(&self, prefs: &Preferences) -> Result<(), String> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(
            &self.path,
            serde_json::to_vec_pretty(prefs).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())
    }

    fn status(&self) -> Result<GpuStatus, String> {
        let prefs = self.prefs.lock().map_err(|e| e.to_string())?;
        let (current, mux_supported, hardware_error) = read_hardware();
        Ok(GpuStatus {
            selected: prefs.mode,
            current,
            supported: current.is_some(),
            mux_supported,
            restart_required: prefs.pending_boot.is_some(),
            error: prefs.error.clone().or(hardware_error),
        })
    }

    fn set(&self, mode: GpuMode) -> Result<GpuStatus, String> {
        let mut prefs = self.prefs.lock().map_err(|e| e.to_string())?;
        if prefs.pending_boot.is_some() {
            return Err("Restart Windows to finish the previous GPU mode change.".into());
        }
        let restart = apply(mode, ac_power()?)?;
        prefs.mode = Some(mode);
        prefs.pending_boot = restart.then(boot_id);
        prefs.error = None;
        self.save(&prefs)
            .map_err(|e| format!("GPU setting applied, but could not save it: {e}"))?;
        drop(prefs);
        self.status()
    }
}

#[tauri::command]
pub async fn get_gpu_mode(config: State<'_, Arc<GpuConfig>>) -> Result<GpuStatus, String> {
    let config = config.inner().clone();
    tauri::async_runtime::spawn_blocking(move || config.status())
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn set_gpu_mode(
    mode: GpuMode,
    config: State<'_, Arc<GpuConfig>>,
) -> Result<GpuStatus, String> {
    let config = config.inner().clone();
    tauri::async_runtime::spawn_blocking(move || config.set(mode))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(any(windows, test))]
fn target(mode: GpuMode, ac: bool) -> GpuMode {
    match mode {
        GpuMode::Auto if ac => GpuMode::Med,
        GpuMode::Auto => GpuMode::Min,
        other => other,
    }
}

#[cfg(windows)]
fn boot_id() -> u64 {
    // SAFETY: this reads the monotonic uptime; no pointers are involved.
    let uptime = unsafe { windows_sys::Win32::System::SystemInformation::GetTickCount64() } / 1000;
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .saturating_sub(uptime)
}
#[cfg(not(windows))]
fn boot_id() -> u64 {
    0
}

#[cfg(windows)]
pub fn ac_power() -> Result<bool, String> {
    use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
    let mut status: SYSTEM_POWER_STATUS = unsafe { std::mem::zeroed() };
    // SAFETY: status points to writable storage of the required size.
    if unsafe { GetSystemPowerStatus(&mut status) } == 0 || status.ACLineStatus > 1 {
        return Err("Windows could not determine the power source.".into());
    }
    Ok(status.ACLineStatus == 1)
}
#[cfg(not(windows))]
pub fn ac_power() -> Result<bool, String> {
    Err("GPU controls currently require Windows and supported ASUS ROG firmware.".into())
}

#[cfg(windows)]
fn read_hardware() -> (Option<GpuMode>, bool, Option<String>) {
    use crate::asus::{Asus, GPU_ECO, GPU_MUX};
    let result = (|| {
        let asus = Asus::open()?;
        let eco = asus.get(GPU_ECO)?;
        let mux = asus.get(GPU_MUX).ok();
        if eco > 1 || mux.is_some_and(|v| v > 1) {
            return Err("The firmware returned an unknown GPU mode.".into());
        }
        Ok((
            Some(if mux == Some(0) {
                GpuMode::Max
            } else if eco == 1 {
                GpuMode::Min
            } else {
                GpuMode::Med
            }),
            mux.is_some(),
            None,
        ))
    })();
    result.unwrap_or_else(|e| (None, false, Some(e)))
}
#[cfg(not(windows))]
fn read_hardware() -> (Option<GpuMode>, bool, Option<String>) {
    (
        None,
        false,
        Some("GPU controls currently require Windows and supported ASUS ROG firmware.".into()),
    )
}

/// True means a MUX change was accepted and a reboot is required.
/// Never disable the dGPU while the current boot may still route its display through it.
#[cfg(windows)]
fn apply(mode: GpuMode, ac: bool) -> Result<bool, String> {
    apply_device(&crate::asus::Asus::open()?, mode, ac)
}

#[cfg(any(windows, test))]
trait GpuDevice {
    fn get(&self, device: u32) -> Result<u32, String>;
    fn set(&self, device: u32, value: u32) -> Result<(), String>;
}
#[cfg(windows)]
impl GpuDevice for crate::asus::Asus {
    fn get(&self, device: u32) -> Result<u32, String> {
        self.get(device)
    }
    fn set(&self, device: u32, value: u32) -> Result<(), String> {
        self.set(device, value)
    }
}

#[cfg(any(windows, test))]
fn apply_device(asus: &impl GpuDevice, mode: GpuMode, ac: bool) -> Result<bool, String> {
    const GPU_ECO: u32 = 0x0009_0020;
    const GPU_MUX: u32 = 0x0009_0016;
    let mode = target(mode, ac);
    let eco = asus.get(GPU_ECO)?;
    if eco > 1 {
        return Err("Unknown GPU firmware state".into());
    }
    let mux = asus.get(GPU_MUX).ok();
    if mux.is_some_and(|value| value > 1) {
        return Err("Unknown GPU MUX state. No GPU setting was changed.".into());
    }
    if mode == GpuMode::Max {
        if mux.is_none() {
            return Err("This device does not have a supported GPU MUX.".into());
        }
        if eco == 1 {
            asus.set(GPU_ECO, 0)?;
            if asus.get(GPU_ECO)? != 0 {
                return Err("Could not enable the dGPU.".into());
            }
        }
        if mux != Some(0) {
            asus.set(GPU_MUX, 0)?;
            if asus.get(GPU_MUX)? != 0 {
                return Err("The firmware did not accept dGPU display mode.".into());
            }
            return Ok(true);
        }
        return Ok(false);
    }
    if mux == Some(0) {
        asus.set(GPU_MUX, 1)?;
        if asus.get(GPU_MUX)? != 1 {
            return Err("The firmware did not accept hybrid display routing.".into());
        }
        return Ok(true);
    }
    let requested = u32::from(mode == GpuMode::Min);
    if eco != requested {
        asus.set(GPU_ECO, requested)?;
        if asus.get(GPU_ECO)? != requested {
            return Err("GPU mode did not change. Close apps using the dGPU and try again.".into());
        }
    }
    Ok(false)
}
#[cfg(not(windows))]
fn apply(_mode: GpuMode, _ac: bool) -> Result<bool, String> {
    Err("GPU controls currently require Windows and supported ASUS ROG firmware.".into())
}
#[cfg(test)]
mod tests;
