//! CPU sustained/boost power limits, persisted and reapplied because firmware PPT values cannot be read back.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};

const MIN_WATTS: u32 = 5;
const MAX_WATTS: u32 = 80;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Limits {
    spl: u32,
    sppt: u32,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
struct Preferences {
    ac: Option<Limits>,
    battery: Option<Limits>,
    #[serde(skip_serializing)]
    spl: Option<u32>,
    #[serde(skip_serializing)]
    sppt: Option<u32>,
    #[serde(skip)]
    error: Option<String>,
}

impl Preferences {
    fn migrate(mut self) -> Self {
        if let (Some(spl), Some(sppt)) = (self.spl.take(), self.sppt.take()) {
            let legacy = Some(Limits { spl, sppt });
            self.ac = self.ac.or(legacy);
            self.battery = self.battery.or(legacy);
        }
        self.ac = self.ac.map(Limits::normalized);
        self.battery = self.battery.map(Limits::normalized);
        self
    }

    #[cfg(any(windows, test))]
    fn profile(&self, ac: bool) -> Option<Limits> {
        if ac { self.ac } else { self.battery }
    }

    fn set_profile(&mut self, ac: bool, limits: Option<Limits>) {
        if ac {
            self.ac = limits;
        } else {
            self.battery = limits;
        }
    }
}

impl Limits {
    fn normalized(self) -> Self {
        let (spl, sppt) = normalize(self.spl, self.sppt);
        Self { spl, sppt }
    }
}

pub struct PowerLimits {
    path: PathBuf,
    prefs: Mutex<Preferences>,
}

#[derive(Serialize)]
pub struct PowerLimitsStatus {
    ac: Option<bool>,
    ac_limits: Option<Limits>,
    battery_limits: Option<Limits>,
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
        let prefs = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Preferences>(&s).ok())
            .unwrap_or_default()
            .migrate();
        let config = Arc::new(Self {
            path,
            prefs: Mutex::new(prefs),
        });
        #[cfg(windows)]
        {
            let worker = config.clone();
            std::thread::spawn(move || {
                let mut watch = crate::config_gate::SourceWatch::default();
                loop {
                    if let Some(ac) = watch.changed(crate::config_gate::enabled(), || {
                        crate::gpu_mode::ac_power().ok()
                    }) && let Ok(mut prefs) = worker.prefs.lock()
                        && let Some(limits) = prefs.profile(ac)
                    {
                        prefs.error = apply(limits.spl, limits.sppt).err();
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
            ac: crate::gpu_mode::ac_power().ok(),
            ac_limits: prefs.ac,
            battery_limits: prefs.battery,
            supported,
            error: prefs.error.clone().or(hardware_error),
        })
    }

    fn set(&self, ac: bool, spl: u32, sppt: u32) -> Result<PowerLimitsStatus, String> {
        let (spl, sppt) = normalize(spl, sppt);
        let mut prefs = self.prefs.lock().map_err(|e| e.to_string())?;
        let live = crate::gpu_mode::ac_power().ok() == Some(ac);
        if live {
            apply(spl, sppt)?;
        }
        prefs.set_profile(ac, Some(Limits { spl, sppt }));
        prefs.error = None;
        self.save(&prefs)
            .map_err(|e| format!("Power limits were not saved: {e}"))?;
        drop(prefs);
        self.status()
    }

    fn reset(&self, ac: bool) -> Result<PowerLimitsStatus, String> {
        let mut prefs = self.prefs.lock().map_err(|e| e.to_string())?;
        prefs.set_profile(ac, None);
        prefs.error = None;
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
    ac: bool,
    spl: u32,
    sppt: u32,
    config: State<'_, Arc<PowerLimits>>,
) -> Result<PowerLimitsStatus, String> {
    crate::config_gate::require()?;
    let config = config.inner().clone();
    tauri::async_runtime::spawn_blocking(move || config.set(ac, spl, sppt))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn reset_power_limits(
    ac: bool,
    config: State<'_, Arc<PowerLimits>>,
) -> Result<PowerLimitsStatus, String> {
    crate::config_gate::require()?;
    let config = config.inner().clone();
    tauri::async_runtime::spawn_blocking(move || config.reset(ac))
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
