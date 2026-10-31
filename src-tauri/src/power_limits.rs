//! CPU sustained/boost power limits, persisted and reapplied because firmware PPT values cannot be read back.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};

const MIN_WATTS: u32 = 5;
const MAX_WATTS: u32 = 80;

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
struct Preferences {
    spl: Option<u32>,
    sppt: Option<u32>,
    #[serde(skip)]
    error: Option<String>,
}

pub struct PowerLimits {
    path: PathBuf,
    prefs: Mutex<Preferences>,
}

#[derive(Serialize)]
pub struct PowerLimitsStatus {
    spl: Option<u32>,
    sppt: Option<u32>,
    supported: bool,
    error: Option<String>,
}

impl PowerLimits {
    pub fn start(app: &AppHandle) -> Result<Arc<Self>, String> {
        let path = app
            .path()
            .app_config_dir()
            .map_err(|e| e.to_string())?
            .join("power-limits.json");
        let mut prefs: Preferences = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        let limits = prefs.spl.zip(prefs.sppt).map(|(a, b)| normalize(a, b));
        prefs.spl = limits.map(|l| l.0);
        prefs.sppt = limits.map(|l| l.1);
        let config = Arc::new(Self {
            path,
            prefs: Mutex::new(prefs),
        });
        #[cfg(windows)]
        {
            let worker = config.clone();
            std::thread::spawn(move || {
                let mut previous = None;
                loop {
                    if let Ok(ac) = crate::gpu_mode::ac_power()
                        && previous != Some(ac)
                    {
                        previous = Some(ac);
                        if let Ok(mut prefs) = worker.prefs.lock()
                            && let (Some(spl), Some(sppt)) = (prefs.spl, prefs.sppt)
                        {
                            prefs.error = apply(spl, sppt).err();
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_secs(5));
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

    fn status(&self) -> Result<PowerLimitsStatus, String> {
        let prefs = self.prefs.lock().map_err(|e| e.to_string())?;
        let (supported, hardware_error) = match supported() {
            Ok(()) => (true, None),
            Err(e) => (false, Some(e)),
        };
        Ok(PowerLimitsStatus {
            spl: prefs.spl,
            sppt: prefs.sppt,
            supported,
            error: prefs.error.clone().or(hardware_error),
        })
    }

    fn set(&self, spl: u32, sppt: u32) -> Result<PowerLimitsStatus, String> {
        let (spl, sppt) = normalize(spl, sppt);
        let mut prefs = self.prefs.lock().map_err(|e| e.to_string())?;
        apply(spl, sppt)?;
        prefs.spl = Some(spl);
        prefs.sppt = Some(sppt);
        prefs.error = None;
        self.save(&prefs)
            .map_err(|e| format!("Power limits applied, but could not save them: {e}"))?;
        drop(prefs);
        self.status()
    }

    fn reset(&self) -> Result<PowerLimitsStatus, String> {
        let mut prefs = self.prefs.lock().map_err(|e| e.to_string())?;
        *prefs = Preferences::default();
        self.save(&prefs)?;
        drop(prefs);
        self.status()
    }
}

#[tauri::command]
pub async fn get_power_limits(
    config: State<'_, Arc<PowerLimits>>,
) -> Result<PowerLimitsStatus, String> {
    let config = config.inner().clone();
    tauri::async_runtime::spawn_blocking(move || config.status())
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn set_power_limits(
    spl: u32,
    sppt: u32,
    config: State<'_, Arc<PowerLimits>>,
) -> Result<PowerLimitsStatus, String> {
    let config = config.inner().clone();
    tauri::async_runtime::spawn_blocking(move || config.set(spl, sppt))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn reset_power_limits(
    config: State<'_, Arc<PowerLimits>>,
) -> Result<PowerLimitsStatus, String> {
    let config = config.inner().clone();
    tauri::async_runtime::spawn_blocking(move || config.reset())
        .await
        .map_err(|e| e.to_string())?
}

fn normalize(spl: u32, sppt: u32) -> (u32, u32) {
    let spl = spl.clamp(MIN_WATTS, MAX_WATTS);
    (spl, sppt.clamp(spl, MAX_WATTS))
}

#[cfg(windows)]
fn supported() -> Result<(), String> {
    crate::asus::Asus::open()?
        .get(crate::asus::PPT_SPL)
        .map(|_| ())
}
#[cfg(not(windows))]
fn supported() -> Result<(), String> {
    Err("Power limits currently require Windows and supported ASUS ROG firmware.".into())
}

#[cfg(windows)]
fn apply(spl: u32, sppt: u32) -> Result<(), String> {
    use crate::asus::{Asus, PPT_FPPT, PPT_SPL, PPT_SPPT};
    let asus = Asus::open()?;
    asus.set(PPT_SPL, spl)?;
    asus.set(PPT_SPPT, sppt)?;
    // FPPT is absent on some models; SPL and SPPT are what matter.
    let _ = asus.set(PPT_FPPT, sppt);
    Ok(())
}
#[cfg(not(windows))]
fn apply(_spl: u32, _sppt: u32) -> Result<(), String> {
    Err("Power limits currently require Windows and supported ASUS ROG firmware.".into())
}
#[cfg(test)]
mod tests;
