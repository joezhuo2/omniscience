//! Battery charge limit via ASUS firmware, persisted and reapplied at launch.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};

const MIN: u32 = 20;
const MAX: u32 = 100;

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct Preferences {
    limit: Option<u32>,
}

pub struct ChargeLimit {
    path: PathBuf,
    prefs: Mutex<Preferences>,
}

#[derive(Serialize)]
pub struct ChargeLimitStatus {
    limit: u32,
    supported: bool,
    error: Option<String>,
}

fn clamp(limit: u32) -> u32 {
    limit.clamp(MIN, MAX)
}

impl ChargeLimit {
    pub fn start(app: &AppHandle) -> Result<Arc<Self>, String> {
        let path = app
            .path()
            .app_config_dir()
            .map_err(|e| e.to_string())?
            .join("charge-limit.json");
        let prefs: Preferences = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        let config = Arc::new(Self {
            path,
            prefs: Mutex::new(prefs),
        });
        #[cfg(windows)]
        {
            let worker = config.clone();
            std::thread::spawn(move || {
                let saved = worker.prefs.lock().ok().and_then(|p| p.limit);
                let Some(limit) = saved.map(clamp) else {
                    return;
                };
                // The ASUS service may not be ready right after login.
                for _ in 0..6 {
                    if write_hardware(limit).is_ok() {
                        return;
                    }
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }
            });
        }
        Ok(config)
    }

    fn status(&self) -> ChargeLimitStatus {
        match read_hardware() {
            Ok(limit) => ChargeLimitStatus {
                limit: clamp(limit),
                supported: true,
                error: None,
            },
            Err(e) => ChargeLimitStatus {
                limit: MAX,
                supported: false,
                error: Some(e),
            },
        }
    }

    fn set(&self, limit: u32) -> Result<ChargeLimitStatus, String> {
        let limit = clamp(limit);
        let mut prefs = self.prefs.lock().map_err(|e| e.to_string())?;
        write_hardware(limit)?;
        prefs.limit = Some(limit);
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(
            &self.path,
            serde_json::to_vec_pretty(&*prefs).map_err(|e| e.to_string())?,
        )
        .map_err(|e| format!("Charge limit applied, but could not save it: {e}"))?;
        drop(prefs);
        Ok(self.status())
    }
}

#[tauri::command]
pub async fn get_charge_limit(
    config: State<'_, Arc<ChargeLimit>>,
) -> Result<ChargeLimitStatus, String> {
    let config = config.inner().clone();
    tauri::async_runtime::spawn_blocking(move || config.status())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_charge_limit(
    limit: u32,
    config: State<'_, Arc<ChargeLimit>>,
) -> Result<ChargeLimitStatus, String> {
    let config = config.inner().clone();
    tauri::async_runtime::spawn_blocking(move || config.set(limit))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(windows)]
fn read_hardware() -> Result<u32, String> {
    crate::asus::Asus::open()?.get(crate::asus::CHARGE_LIMIT)
}
#[cfg(not(windows))]
fn read_hardware() -> Result<u32, String> {
    Err("Charge limit currently requires Windows and supported ASUS firmware.".into())
}

#[cfg(windows)]
fn write_hardware(limit: u32) -> Result<(), String> {
    let asus = crate::asus::Asus::open()?;
    asus.set(crate::asus::CHARGE_LIMIT, limit)?;
    if asus.get(crate::asus::CHARGE_LIMIT)? != limit {
        return Err("The firmware did not accept the charge limit.".into());
    }
    Ok(())
}
#[cfg(not(windows))]
fn write_hardware(_limit: u32) -> Result<(), String> {
    Err("Charge limit currently requires Windows and supported ASUS firmware.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_to_supported_range() {
        assert_eq!(clamp(0), 20);
        assert_eq!(clamp(60), 60);
        assert_eq!(clamp(250), 100);
    }

    #[test]
    fn preferences_default_to_unset_and_round_trip() {
        let empty: Preferences = serde_json::from_str("{}").unwrap();
        assert_eq!(empty.limit, None);
        let saved = serde_json::to_string(&Preferences { limit: Some(80) }).unwrap();
        assert_eq!(
            serde_json::from_str::<Preferences>(&saved).unwrap().limit,
            Some(80)
        );
    }
}
