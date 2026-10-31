//! Built-in display resolution and refresh-rate controls with opt-in AC/battery automation.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
struct Mode {
    width: u32,
    height: u32,
    hz: u32,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
struct Preferences {
    auto: bool,
    rate: Option<u32>,
    #[serde(skip)]
    error: Option<String>,
}

pub struct DisplayConfig {
    path: PathBuf,
    prefs: Mutex<Preferences>,
}

#[derive(Serialize)]
pub struct DisplayStatus {
    device: Option<String>,
    current: Option<Mode>,
    resolutions: Vec<[u32; 2]>,
    rates: Vec<u32>,
    auto: bool,
    supported: bool,
    error: Option<String>,
}

struct Snapshot {
    device: String,
    current: Mode,
    modes: Vec<Mode>,
}

impl DisplayConfig {
    pub fn start(app: &AppHandle) -> Result<Arc<Self>, String> {
        let path = app
            .path()
            .app_config_dir()
            .map_err(|e| e.to_string())?
            .join("display.json");
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
                loop {
                    if let Ok(mut prefs) = worker.prefs.lock()
                        && prefs.auto
                    {
                        match crate::gpu_mode::ac_power() {
                            Ok(ac) if previous != Some(ac) => {
                                previous = Some(ac);
                                prefs.error = read().and_then(|snap| apply_auto(&snap, ac)).err();
                            }
                            Err(e) => prefs.error = Some(e),
                            _ => {}
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

    fn status(&self) -> Result<DisplayStatus, String> {
        let prefs = self.prefs.lock().map_err(|e| e.to_string())?;
        Ok(match read() {
            Ok(snap) => DisplayStatus {
                resolutions: resolutions(&snap.modes),
                rates: rates_at(&snap.modes, snap.current.width, snap.current.height),
                device: Some(snap.device),
                current: Some(snap.current),
                auto: prefs.auto,
                supported: true,
                error: prefs.error.clone(),
            },
            Err(e) => DisplayStatus {
                device: None,
                current: None,
                resolutions: vec![],
                rates: vec![],
                auto: prefs.auto,
                supported: false,
                error: Some(e),
            },
        })
    }

    fn set_refresh(&self, hz: Option<u32>) -> Result<DisplayStatus, String> {
        let mut prefs = self.prefs.lock().map_err(|e| e.to_string())?;
        let snap = read()?;
        match hz {
            None => apply_auto(&snap, crate::gpu_mode::ac_power()?)?,
            Some(hz) => apply_mode(&snap, Mode { hz, ..snap.current })?,
        }
        prefs.auto = hz.is_none();
        prefs.rate = hz.or(prefs.rate);
        prefs.error = None;
        self.save(&prefs)
            .map_err(|e| format!("Display setting applied, but could not save it: {e}"))?;
        drop(prefs);
        self.status()
    }

    fn set_resolution(&self, width: u32, height: u32) -> Result<DisplayStatus, String> {
        let prefs = self.prefs.lock().map_err(|e| e.to_string())?;
        let snap = read()?;
        let rates = rates_at(&snap.modes, width, height);
        let ac = if prefs.auto {
            Some(crate::gpu_mode::ac_power()?)
        } else {
            None
        };
        let hz = resolution_rate(&rates, snap.current.hz, ac)
            .ok_or_else(|| format!("{width}×{height} is not available."))?;
        apply_mode(&snap, Mode { width, height, hz })?;
        drop(prefs);
        self.status()
    }
}

#[tauri::command]
pub async fn get_display_mode(
    config: State<'_, Arc<DisplayConfig>>,
) -> Result<DisplayStatus, String> {
    let config = config.inner().clone();
    tauri::async_runtime::spawn_blocking(move || config.status())
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn set_display_refresh(
    hz: Option<u32>,
    config: State<'_, Arc<DisplayConfig>>,
) -> Result<DisplayStatus, String> {
    let config = config.inner().clone();
    tauri::async_runtime::spawn_blocking(move || config.set_refresh(hz))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn set_display_resolution(
    width: u32,
    height: u32,
    config: State<'_, Arc<DisplayConfig>>,
) -> Result<DisplayStatus, String> {
    let config = config.inner().clone();
    tauri::async_runtime::spawn_blocking(move || config.set_resolution(width, height))
        .await
        .map_err(|e| e.to_string())?
}

fn resolutions(modes: &[Mode]) -> Vec<[u32; 2]> {
    let mut list: Vec<[u32; 2]> = modes.iter().map(|m| [m.width, m.height]).collect();
    list.sort_unstable_by(|a, b| b.cmp(a));
    list.dedup();
    list
}

fn rates_at(modes: &[Mode], width: u32, height: u32) -> Vec<u32> {
    let mut list: Vec<u32> = modes
        .iter()
        .filter(|m| m.width == width && m.height == height)
        .map(|m| m.hz)
        .collect();
    list.sort_unstable();
    list.dedup();
    list
}

fn auto_rate(rates: &[u32], ac: bool) -> Option<u32> {
    if ac {
        rates.iter().max()
    } else {
        rates.iter().find(|&&hz| hz >= 60).or(rates.iter().max())
    }
    .copied()
}

fn resolution_rate(rates: &[u32], current: u32, auto_ac: Option<bool>) -> Option<u32> {
    match auto_ac {
        Some(ac) => auto_rate(rates, ac),
        None if rates.contains(&current) => Some(current),
        None => rates.iter().max().copied(),
    }
}

fn apply_auto(snap: &Snapshot, ac: bool) -> Result<(), String> {
    let rates = rates_at(&snap.modes, snap.current.width, snap.current.height);
    match auto_rate(&rates, ac) {
        Some(hz) if hz != snap.current.hz => apply_mode(snap, Mode { hz, ..snap.current }),
        _ => Ok(()),
    }
}

fn apply_mode(snap: &Snapshot, mode: Mode) -> Result<(), String> {
    if !snap.modes.contains(&mode) {
        return Err("That display mode is not available.".into());
    }
    change(mode)
}

#[cfg(windows)]
fn read() -> Result<Snapshot, String> {
    win::read()
}
#[cfg(windows)]
fn change(mode: Mode) -> Result<(), String> {
    win::change(mode)
}
#[cfg(not(windows))]
fn read() -> Result<Snapshot, String> {
    Err("Display controls currently require Windows.".into())
}
#[cfg(not(windows))]
fn change(_mode: Mode) -> Result<(), String> {
    Err("Display controls currently require Windows.".into())
}

#[cfg(windows)]
mod win {
    use super::{Mode, Snapshot};
    use windows_sys::Win32::Devices::Display::*;
    use windows_sys::Win32::Graphics::Gdi::*;

    fn wide_to_string(w: &[u16]) -> String {
        String::from_utf16_lossy(&w[..w.iter().position(|&c| c == 0).unwrap_or(w.len())])
    }

    fn internal_names() -> Vec<String> {
        let (mut np, mut nm) = (0u32, 0u32);
        // SAFETY: out-pointers reference valid u32 locals.
        if unsafe { GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut np, &mut nm) } != 0 {
            return vec![];
        }
        let mut paths: Vec<DISPLAYCONFIG_PATH_INFO> =
            vec![unsafe { std::mem::zeroed() }; np as usize];
        let mut modes: Vec<DISPLAYCONFIG_MODE_INFO> =
            vec![unsafe { std::mem::zeroed() }; nm as usize];
        // SAFETY: buffers are sized by GetDisplayConfigBufferSizes.
        if unsafe {
            QueryDisplayConfig(
                QDC_ONLY_ACTIVE_PATHS,
                &mut np,
                paths.as_mut_ptr(),
                &mut nm,
                modes.as_mut_ptr(),
                std::ptr::null_mut(),
            )
        } != 0
        {
            return vec![];
        }
        paths[..np as usize]
            .iter()
            .filter(|p| {
                p.targetInfo.outputTechnology == DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INTERNAL
                    || p.targetInfo.outputTechnology
                        == DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EMBEDDED
            })
            .filter_map(|p| {
                let mut name: DISPLAYCONFIG_SOURCE_DEVICE_NAME = unsafe { std::mem::zeroed() };
                name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
                name.header.size = std::mem::size_of_val(&name) as u32;
                name.header.adapterId = p.sourceInfo.adapterId;
                name.header.id = p.sourceInfo.id;
                // SAFETY: header describes a correctly sized source-name request.
                (unsafe { DisplayConfigGetDeviceInfo(&mut name.header) } == 0)
                    .then(|| wide_to_string(&name.viewGdiDeviceName))
            })
            .collect()
    }

    fn target() -> Result<(Vec<u16>, String), String> {
        let internal = internal_names();
        let mut best: Option<(u8, DISPLAY_DEVICEW)> = None;
        for i in 0.. {
            let mut dd: DISPLAY_DEVICEW = unsafe { std::mem::zeroed() };
            dd.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;
            // SAFETY: dd is initialised with its size.
            if unsafe { EnumDisplayDevicesW(std::ptr::null(), i, &mut dd, 0) } == 0 {
                break;
            }
            if dd.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP == 0 {
                continue;
            }
            let rank = if internal.contains(&wide_to_string(&dd.DeviceName)) {
                2
            } else if dd.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE != 0 {
                1
            } else {
                0
            };
            if best.is_none_or(|(r, _)| rank > r) {
                best = Some((rank, dd));
            }
        }
        let (_, dd) = best.ok_or("No active display was found.")?;
        let mut name: Vec<u16> = dd
            .DeviceName
            .iter()
            .copied()
            .take_while(|&c| c != 0)
            .collect();
        name.push(0);
        let mut mon: DISPLAY_DEVICEW = unsafe { std::mem::zeroed() };
        mon.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;
        // SAFETY: name is NUL-terminated and mon is initialised with its size.
        let label = if unsafe { EnumDisplayDevicesW(name.as_ptr(), 0, &mut mon, 0) } != 0 {
            wide_to_string(&mon.DeviceString)
        } else {
            wide_to_string(&dd.DeviceString)
        };
        Ok((name, label))
    }

    fn enum_mode(name: &[u16], index: u32) -> Option<DEVMODEW> {
        let mut dm: DEVMODEW = unsafe { std::mem::zeroed() };
        dm.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
        // SAFETY: name is NUL-terminated and dm is initialised with its size.
        (unsafe { EnumDisplaySettingsW(name.as_ptr(), index, &mut dm) } != 0).then_some(dm)
    }

    fn orientation(dm: &DEVMODEW) -> u32 {
        // SAFETY: the display variant of the union is the one populated for monitors.
        unsafe { dm.Anonymous1.Anonymous2.dmDisplayOrientation }
    }

    fn to_mode(dm: &DEVMODEW) -> Mode {
        Mode {
            width: dm.dmPelsWidth,
            height: dm.dmPelsHeight,
            hz: dm.dmDisplayFrequency,
        }
    }

    fn compatible(name: &[u16], cur: &DEVMODEW) -> Vec<DEVMODEW> {
        (0..)
            .map_while(|i| enum_mode(name, i))
            .filter(|dm| {
                dm.dmBitsPerPel == cur.dmBitsPerPel
                    && orientation(dm) == orientation(cur)
                    && dm.dmDisplayFrequency > 1
            })
            .collect()
    }

    pub fn read() -> Result<Snapshot, String> {
        let (name, device) = target()?;
        let cur = enum_mode(&name, ENUM_CURRENT_SETTINGS)
            .ok_or("Could not read the current display mode.")?;
        let mut modes: Vec<Mode> = compatible(&name, &cur).iter().map(to_mode).collect();
        modes.sort_unstable_by_key(|m| (m.width, m.height, m.hz));
        modes.dedup();
        Ok(Snapshot {
            device,
            current: to_mode(&cur),
            modes,
        })
    }

    pub fn change(mode: Mode) -> Result<(), String> {
        let (name, _) = target()?;
        let cur = enum_mode(&name, ENUM_CURRENT_SETTINGS)
            .ok_or("Could not read the current display mode.")?;
        let dm = compatible(&name, &cur)
            .into_iter()
            .find(|dm| to_mode(dm) == mode)
            .ok_or("That display mode is not available.")?;
        for flags in [CDS_TEST, CDS_UPDATEREGISTRY] {
            // SAFETY: name is NUL-terminated and dm is an enumerated mode.
            let result = unsafe {
                ChangeDisplaySettingsExW(
                    name.as_ptr(),
                    &dm,
                    std::ptr::null_mut(),
                    flags,
                    std::ptr::null(),
                )
            };
            if result != DISP_CHANGE_SUCCESSFUL {
                return Err(format!(
                    "Windows rejected the display mode (code {result})."
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
