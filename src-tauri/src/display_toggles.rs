use serde::Serialize;

#[derive(Default, Serialize)]
pub struct DisplayToggles {
    hdr: Option<bool>,
    hdr_supported: bool,
    night_light: Option<bool>,
    brightness: Option<u8>,
    error: Option<String>,
}

#[cfg(windows)]
fn read() -> DisplayToggles {
    let color = win::hdr_state();
    DisplayToggles {
        hdr: color.and_then(|(supported, on)| supported.then_some(on)),
        hdr_supported: color.is_some_and(|(supported, _)| supported),
        night_light: win::night_light(),
        brightness: win::brightness(),
        error: None,
    }
}
#[cfg(not(windows))]
fn read() -> DisplayToggles {
    DisplayToggles::default()
}

#[cfg(windows)]
fn write_hdr(on: bool) -> Result<(), String> {
    win::set_hdr(on)
}
#[cfg(not(windows))]
fn write_hdr(_on: bool) -> Result<(), String> {
    Err("HDR control is not supported on this device.".into())
}

#[cfg(windows)]
fn write_night_light(on: bool) -> Result<(), String> {
    win::set_night_light(on)
}
#[cfg(not(windows))]
fn write_night_light(_on: bool) -> Result<(), String> {
    Err("Night light control is not supported on this device.".into())
}

#[cfg(windows)]
fn write_brightness(percent: u8) -> Result<(), String> {
    win::set_brightness(percent)
}
#[cfg(not(windows))]
fn write_brightness(_percent: u8) -> Result<(), String> {
    Err("Brightness control is not supported on this device.".into())
}

#[cfg(windows)]
mod win {
    use std::time::Duration;
    use windows_sys::Win32::Devices::Display::*;
    use windows_sys::Win32::Foundation::LUID;
    use windows_sys::Win32::System::Registry::*;

    const NIGHT_LIGHT_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\CloudStore\Store\DefaultAccount\Current\default$windows.data.bluelightreduction.bluelightreductionstate\windows.data.bluelightreduction.bluelightreductionstate";
    const NIGHT_LIGHT_VALUE: &str = "Data";
    const MAGIC: [u8; 4] = [0x43, 0x42, 0x01, 0x00];
    const OFF_LEN: usize = 41;
    const STATE_AT: usize = 18;
    const MARKER_AT: usize = 23;
    const MARKER: [u8; 2] = [0x10, 0x00];
    const OFF_STATE: u8 = 0x13;
    const ON_STATE: u8 = 0x15;
    const STAMP: std::ops::Range<usize> = 10..15;

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }

    pub fn parse_night_light(data: &[u8]) -> Option<bool> {
        let framed = data.len() > MARKER_AT + MARKER.len()
            && data[..4] == MAGIC
            && data[STATE_AT + 1..MARKER_AT] == MAGIC;
        match (framed, data.get(STATE_AT)) {
            (true, Some(&OFF_STATE)) if data.len() == OFF_LEN => Some(false),
            (true, Some(&ON_STATE))
                if data.len() == OFF_LEN + MARKER.len()
                    && data[MARKER_AT..MARKER_AT + MARKER.len()] == MARKER =>
            {
                Some(true)
            }
            _ => None,
        }
    }

    pub fn toggled_night_light(data: &[u8], on: bool) -> Option<Vec<u8>> {
        let current = parse_night_light(data)?;
        if current == on {
            return Some(data.to_vec());
        }
        let mut out = data[..MARKER_AT].to_vec();
        if on {
            out.extend_from_slice(&MARKER);
            out.extend_from_slice(&data[MARKER_AT..]);
        } else {
            out.extend_from_slice(&data[MARKER_AT + MARKER.len()..]);
        }
        out[STATE_AT] = if on { ON_STATE } else { OFF_STATE };
        for byte in &mut out[STAMP] {
            if *byte == 0xff {
                *byte = 0;
            } else {
                *byte += 1;
                break;
            }
        }
        Some(out)
    }

    fn read_blob() -> Option<Vec<u8>> {
        let key = wide(NIGHT_LIGHT_KEY);
        let value = wide(NIGHT_LIGHT_VALUE);
        let mut size = 0u32;
        // SAFETY: key and value are NUL-terminated; a null buffer only queries the size.
        let status = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                value.as_ptr(),
                RRF_RT_REG_BINARY,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut size,
            )
        };
        if status != 0 || size == 0 {
            return None;
        }
        let mut buf = vec![0u8; size as usize];
        // SAFETY: buf has the capacity reported in size.
        let status = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                value.as_ptr(),
                RRF_RT_REG_BINARY,
                std::ptr::null_mut(),
                buf.as_mut_ptr().cast(),
                &mut size,
            )
        };
        buf.truncate(size as usize);
        (status == 0).then_some(buf)
    }

    pub fn night_light() -> Option<bool> {
        parse_night_light(&read_blob()?)
    }

    pub fn set_night_light(on: bool) -> Result<(), String> {
        let blob = read_blob().ok_or("Night light settings were not found on this device.")?;
        let new = toggled_night_light(&blob, on)
            .ok_or("Night light settings have an unrecognised format.")?;
        if new == blob {
            return Ok(());
        }
        let key = wide(NIGHT_LIGHT_KEY);
        let value = wide(NIGHT_LIGHT_VALUE);
        // SAFETY: key and value are NUL-terminated and new is valid for its length.
        let status = unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                value.as_ptr(),
                REG_BINARY,
                new.as_ptr().cast(),
                new.len() as u32,
            )
        };
        if status != 0 {
            return Err(format!(
                "Could not change night light: {}",
                std::io::Error::from_raw_os_error(status as i32)
            ));
        }
        Ok(())
    }

    fn primary_target() -> Option<(LUID, u32)> {
        let (mut np, mut nm) = (0u32, 0u32);
        // SAFETY: out-pointers reference valid u32 locals.
        if unsafe { GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut np, &mut nm) } != 0 {
            return None;
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
            return None;
        }
        let source = modes[..nm as usize].iter().find(|m| {
            // SAFETY: the source variant is populated when infoType is SOURCE.
            m.infoType == DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE
                && unsafe {
                    let p = m.Anonymous.sourceMode.position;
                    p.x == 0 && p.y == 0
                }
        })?;
        paths[..np as usize]
            .iter()
            .find(|p| {
                p.sourceInfo.id == source.id
                    && p.sourceInfo.adapterId.LowPart == source.adapterId.LowPart
                    && p.sourceInfo.adapterId.HighPart == source.adapterId.HighPart
            })
            .map(|p| (p.targetInfo.adapterId, p.targetInfo.id))
    }

    pub fn hdr_state() -> Option<(bool, bool)> {
        let (adapter, id) = primary_target()?;
        let mut info: DISPLAYCONFIG_GET_ADVANCED_COLOR_INFO = unsafe { std::mem::zeroed() };
        info.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO;
        info.header.size = std::mem::size_of_val(&info) as u32;
        info.header.adapterId = adapter;
        info.header.id = id;
        // SAFETY: header describes a correctly sized advanced-color request.
        if unsafe { DisplayConfigGetDeviceInfo(&mut info.header) } != 0 {
            return None;
        }
        // SAFETY: both union variants are plain u32 bit fields.
        let bits = unsafe { info.Anonymous.value };
        Some((bits & 1 != 0, bits & 2 != 0))
    }

    pub fn set_hdr(on: bool) -> Result<(), String> {
        if !hdr_state().is_some_and(|(supported, _)| supported) {
            return Err("This display does not support HDR.".into());
        }
        let (adapter, id) = primary_target().ok_or("No active display was found.")?;
        let mut state: DISPLAYCONFIG_SET_ADVANCED_COLOR_STATE = unsafe { std::mem::zeroed() };
        state.header.r#type = DISPLAYCONFIG_DEVICE_INFO_SET_ADVANCED_COLOR_STATE;
        state.header.size = std::mem::size_of_val(&state) as u32;
        state.header.adapterId = adapter;
        state.header.id = id;
        state.Anonymous.value = on as u32;
        // SAFETY: header describes a correctly sized advanced-color state request.
        let status = unsafe { DisplayConfigSetDeviceInfo(&state.header) };
        if status != 0 {
            return Err(format!(
                "Could not change HDR: {}",
                std::io::Error::from_raw_os_error(status)
            ));
        }
        Ok(())
    }

    pub fn brightness() -> Option<u8> {
        crate::system_command::powershell(
            "Get-CimInstance -Namespace root/wmi -ClassName WmiMonitorBrightness | Select-Object -First 1 -ExpandProperty CurrentBrightness",
            Duration::from_secs(10),
        )
        .ok()?
        .parse::<u8>()
        .ok()
        .map(|v| v.min(100))
    }

    pub fn set_brightness(percent: u8) -> Result<(), String> {
        crate::system_command::powershell(
            &format!(
                "Get-CimInstance -Namespace root/wmi -ClassName WmiMonitorBrightnessMethods | Select-Object -First 1 | Invoke-CimMethod -MethodName WmiSetBrightness -Arguments @{{Timeout=[uint32]1;Brightness=[byte]{percent}}} | Out-Null"
            ),
            Duration::from_secs(10),
        )
        .map(|_| ())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn off_blob() -> Vec<u8> {
            let mut b = vec![0x43, 0x42, 0x01, 0x00, 0x0a, 0x02, 0x01, 0x00, 0x2a, 0x06];
            b.extend([0xa1, 0xb2, 0xc3, 0xd4, 0x05]);
            b.extend([0x2a, 0x2b, 0x0e, 0x13, 0x43, 0x42, 0x01, 0x00]);
            b.extend([0xc2, 0x0a, 0x00, 0xca, 0x14, 0x0e, 0x15, 0x00]);
            b.extend([0xca, 0x32, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
            assert_eq!(b.len(), OFF_LEN);
            b
        }

        fn on_blob() -> Vec<u8> {
            let mut b = off_blob();
            b[10..15].copy_from_slice(&[0xa2, 0xb2, 0xc3, 0xd4, 0x05]);
            b[STATE_AT] = ON_STATE;
            b.splice(MARKER_AT..MARKER_AT, MARKER);
            b
        }

        #[test]
        fn reads_state_from_blob() {
            assert_eq!(parse_night_light(&off_blob()), Some(false));
            assert_eq!(parse_night_light(&on_blob()), Some(true));
        }

        #[test]
        fn rejects_unknown_structure() {
            let short = [
                0x43, 0x42, 0x01, 0x00, 0x0a, 0x02, 0x01, 0x00, 0x2a, 0x2a, 0, 0, 0,
            ];
            assert_eq!(parse_night_light(&short), None);
            assert_eq!(parse_night_light(&[]), None);
            let mut bad = off_blob();
            bad[STATE_AT] = 0x99;
            assert_eq!(parse_night_light(&bad), None);
            assert_eq!(toggled_night_light(&short, true), None);
        }

        #[test]
        fn toggles_both_ways_and_bumps_the_stamp() {
            let on = toggled_night_light(&off_blob(), true).unwrap();
            assert_eq!(parse_night_light(&on), Some(true));
            assert_eq!(on[10], 0xa2);
            assert_eq!(on, on_blob());
            let off = toggled_night_light(&on_blob(), false).unwrap();
            assert_eq!(parse_night_light(&off), Some(false));
            assert_eq!(off[10], 0xa3);
            assert_eq!(off[11..], off_blob()[11..]);
        }

        #[test]
        fn same_state_is_left_untouched_and_stamp_carries() {
            assert_eq!(toggled_night_light(&off_blob(), false).unwrap(), off_blob());
            let mut blob = off_blob();
            blob[10..12].copy_from_slice(&[0xff, 0x01]);
            let on = toggled_night_light(&blob, true).unwrap();
            assert_eq!(on[10..12], [0x00, 0x02]);
        }

        #[test]
        #[ignore]
        fn live_read_only() {
            println!(
                "hdr: {:?} night light: {:?} brightness: {:?}",
                hdr_state(),
                night_light(),
                brightness()
            );
        }
    }
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
