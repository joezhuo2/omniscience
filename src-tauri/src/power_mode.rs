//! Windows power mode overlay and opt-in AC/battery automation.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PowerMode {
    Auto,
    Efficiency,
    Balanced,
    Performance,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
struct Preferences {
    mode: Option<PowerMode>,
    #[serde(skip)]
    error: Option<String>,
}

pub struct PowerConfig {
    path: PathBuf,
    prefs: Mutex<Preferences>,
}

#[derive(Serialize)]
pub struct PowerStatus {
    selected: Option<PowerMode>,
    current: Option<PowerMode>,
    supported: bool,
    error: Option<String>,
}

#[cfg(any(windows, test))]
const EFFICIENCY: &str = "961cc777-2547-4f9d-8174-7d86181b8a7a";
#[cfg(any(windows, test))]
const BALANCED: &str = "00000000-0000-0000-0000-000000000000";
#[cfg(any(windows, test))]
const PERFORMANCE: &str = "ded574b5-45a0-4f42-8737-46345c09c238";

impl PowerConfig {
    pub fn start(app: &AppHandle) -> Result<Arc<Self>, String> {
        let path = app
            .path()
            .app_config_dir()
            .map_err(|e| e.to_string())?
            .join("power-mode.json");
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
                let mut previous = None;
                let mut initial = true;
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    let Ok(mut prefs) = worker.prefs.lock() else {
                        continue;
                    };
                    if let Some(mode) = prefs.mode
                        && (initial || mode == PowerMode::Auto)
                    {
                        match crate::gpu_mode::ac_power() {
                            Ok(ac) if initial || previous != Some(ac) => {
                                previous = Some(ac);
                                prefs.error = apply(target(mode, ac)).err();
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

    fn status(&self) -> Result<PowerStatus, String> {
        let prefs = self.prefs.lock().map_err(|e| e.to_string())?;
        let (current, supported, hardware_error) = read_current();
        Ok(PowerStatus {
            selected: prefs.mode,
            current,
            supported,
            error: prefs.error.clone().or(hardware_error),
        })
    }

    fn set(&self, mode: PowerMode) -> Result<PowerStatus, String> {
        let mut prefs = self.prefs.lock().map_err(|e| e.to_string())?;
        apply(target(mode, crate::gpu_mode::ac_power()?))?;
        prefs.mode = Some(mode);
        prefs.error = None;
        self.save(&prefs)
            .map_err(|e| format!("Power mode applied, but could not save it: {e}"))?;
        drop(prefs);
        self.status()
    }
}

#[tauri::command]
pub async fn get_power_mode(config: State<'_, Arc<PowerConfig>>) -> Result<PowerStatus, String> {
    let config = config.inner().clone();
    tauri::async_runtime::spawn_blocking(move || config.status())
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn set_power_mode(
    mode: PowerMode,
    config: State<'_, Arc<PowerConfig>>,
) -> Result<PowerStatus, String> {
    let config = config.inner().clone();
    tauri::async_runtime::spawn_blocking(move || config.set(mode))
        .await
        .map_err(|e| e.to_string())?
}

fn target(mode: PowerMode, ac: bool) -> PowerMode {
    match mode {
        PowerMode::Auto if ac => PowerMode::Performance,
        PowerMode::Auto => PowerMode::Efficiency,
        other => other,
    }
}

#[cfg(any(windows, test))]
fn guid_of(mode: PowerMode) -> &'static str {
    match mode {
        PowerMode::Efficiency => EFFICIENCY,
        PowerMode::Performance => PERFORMANCE,
        _ => BALANCED,
    }
}

#[cfg(any(windows, test))]
fn mode_of(guid: &str) -> Option<PowerMode> {
    [
        (EFFICIENCY, PowerMode::Efficiency),
        (BALANCED, PowerMode::Balanced),
        (PERFORMANCE, PowerMode::Performance),
    ]
    .into_iter()
    .find(|(g, _)| guid.eq_ignore_ascii_case(g))
    .map(|(_, mode)| mode)
}

#[cfg(windows)]
mod win {
    use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
    use windows_sys::core::GUID;

    // powrprof.lib does not export these, so resolve them at runtime.
    fn symbol(name: &[u8]) -> Option<unsafe extern "system" fn() -> isize> {
        let lib: Vec<u16> = "powrprof.dll\0".encode_utf16().collect();
        // SAFETY: both strings are NUL-terminated.
        unsafe {
            let module = LoadLibraryW(lib.as_ptr());
            if module.is_null() {
                return None;
            }
            GetProcAddress(module, name.as_ptr())
        }
    }

    pub fn get_effective(scheme: &mut GUID) -> Result<u32, String> {
        let f = symbol(b"PowerGetEffectiveOverlayScheme\0")
            .ok_or("Windows power modes are unavailable on this version of Windows.")?;
        // SAFETY: the export has this signature on Windows 10 1709+.
        let f: unsafe extern "system" fn(*mut GUID) -> u32 = unsafe { std::mem::transmute(f) };
        Ok(unsafe { f(scheme) })
    }

    pub fn set_active(scheme: GUID) -> Result<u32, String> {
        let f = symbol(b"PowerSetActiveOverlayScheme\0")
            .ok_or("Windows power modes are unavailable on this version of Windows.")?;
        // SAFETY: the export has this signature on Windows 10 1709+.
        let f: unsafe extern "system" fn(GUID) -> u32 = unsafe { std::mem::transmute(f) };
        Ok(unsafe { f(scheme) })
    }
}

#[cfg(windows)]
fn guid_string(g: &windows_sys::core::GUID) -> String {
    let d = g.data4;
    format!(
        "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        g.data1, g.data2, g.data3, d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7]
    )
}

#[cfg(windows)]
fn parse_guid(s: &str) -> windows_sys::core::GUID {
    let n = u128::from_str_radix(&s.replace('-', ""), 16).unwrap_or_default();
    windows_sys::core::GUID {
        data1: (n >> 96) as u32,
        data2: (n >> 80) as u16,
        data3: (n >> 64) as u16,
        data4: (n as u64).to_be_bytes(),
    }
}

#[cfg(windows)]
fn read_current() -> (Option<PowerMode>, bool, Option<String>) {
    let mut guid = parse_guid(BALANCED);
    let code = match win::get_effective(&mut guid) {
        Ok(code) => code,
        Err(e) => return (None, false, Some(e)),
    };
    if code != 0 {
        let error = format!("Windows power modes are unavailable (error {code}).");
        return (None, false, Some(error));
    }
    let raw = guid_string(&guid);
    match mode_of(&raw) {
        Some(mode) => (Some(mode), true, None),
        None => (
            None,
            true,
            Some(format!("Windows is using an unknown power mode ({raw}).")),
        ),
    }
}
#[cfg(not(windows))]
fn read_current() -> (Option<PowerMode>, bool, Option<String>) {
    (
        None,
        false,
        Some("Power mode controls currently require Windows 11.".into()),
    )
}

#[cfg(windows)]
fn apply(mode: PowerMode) -> Result<(), String> {
    let code = win::set_active(parse_guid(guid_of(mode)))?;
    if code != 0 {
        return Err(format!("Windows refused the power mode (error {code})."));
    }
    if read_current().0 != Some(mode) {
        return Err("Windows did not apply the requested power mode.".into());
    }
    Ok(())
}
#[cfg(not(windows))]
fn apply(_mode: PowerMode) -> Result<(), String> {
    Err("Power mode controls currently require Windows 11.".into())
}

#[cfg(test)]
mod tests;
