//! Windows energy saver: live state, the active plan's battery threshold (ESBATTTHRESHOLD) and a manual override.
use serde::Serialize;
use tauri::command;

#[derive(Serialize)]
pub struct BatterySaverStatus {
    on: Option<bool>,
    threshold: Option<u32>,
    supported: bool,
    error: Option<String>,
}

fn validate(threshold: u32) -> Result<u32, String> {
    if threshold == 0 || ((20..=100).contains(&threshold) && threshold.is_multiple_of(10)) {
        Ok(threshold)
    } else {
        Err("Battery saver threshold must be 0 (Off) or 20-100% in steps of 10.".into())
    }
}

#[cfg(windows)]
mod win {
    use super::*;
    use crate::cpu_boost::win::{Scheme, check};
    use windows_sys::Win32::System::Power::{
        GetSystemPowerStatus, PowerReadDCValueIndex, PowerSetActiveScheme, PowerWriteDCValueIndex,
        SYSTEM_POWER_STATUS,
    };
    use windows_sys::core::GUID;

    const SUB_ENERGYSAVER: GUID = GUID::from_u128(0xde830923_a562_41af_a086_e3a2c6bad2da);
    const ESBATTTHRESHOLD: GUID = GUID::from_u128(0xe69653ca_cf7f_4f05_aa73_cb833fa90ad4);
    // Undocumented: the WNF state the Windows Settings energy saver toggle publishes (1 = off, 2 = on).
    const ENERGY_SAVER_STATE: u64 = 0x41c6_013d_a3bc_3075;

    fn on() -> Result<bool, String> {
        let mut status = SYSTEM_POWER_STATUS::default();
        // SAFETY: status points to writable storage of the required size.
        if unsafe { GetSystemPowerStatus(&mut status) } == 0 {
            return Err("Windows could not report the battery saver state.".into());
        }
        Ok(status.SystemStatusFlag == 1)
    }

    pub fn read() -> Result<BatterySaverStatus, String> {
        let scheme = Scheme::active()?;
        let mut threshold = 0u32;
        // SAFETY: scheme and the GUID constants are valid; threshold is a writable u32.
        check(
            unsafe {
                PowerReadDCValueIndex(
                    std::ptr::null_mut(),
                    scheme.0,
                    &SUB_ENERGYSAVER,
                    &ESBATTTHRESHOLD,
                    &mut threshold,
                )
            },
            "Reading the battery saver threshold",
        )?;
        Ok(BatterySaverStatus {
            on: Some(on()?),
            threshold: Some(threshold),
            supported: true,
            error: None,
        })
    }

    pub fn write_threshold(threshold: u32) -> Result<(), String> {
        let scheme = Scheme::active()?;
        // SAFETY: scheme and the GUID constants are valid for the duration of the calls.
        unsafe {
            check(
                PowerWriteDCValueIndex(
                    std::ptr::null_mut(),
                    scheme.0,
                    &SUB_ENERGYSAVER,
                    &ESBATTTHRESHOLD,
                    threshold,
                ),
                "Writing the battery saver threshold",
            )?;
            check(
                PowerSetActiveScheme(std::ptr::null_mut(), scheme.0),
                "Applying the power plan",
            )
        }
    }

    pub fn set(on: bool) -> Result<(), String> {
        use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
        let ntdll: Vec<u16> = "ntdll.dll\0".encode_utf16().collect();
        // SAFETY: both strings are NUL-terminated and ntdll is loaded in every process.
        let publish = unsafe {
            GetProcAddress(
                GetModuleHandleW(ntdll.as_ptr()),
                c"RtlPublishWnfStateData".as_ptr().cast(),
            )
        }
        .ok_or("Windows does not expose the battery saver switch.")?;
        // SAFETY: the export takes (WNF_STATE_NAME, type id, buffer, length, scope) and returns an NTSTATUS.
        let publish: unsafe extern "system" fn(
            u64,
            *const GUID,
            *const u32,
            u32,
            *const u8,
        ) -> i32 = unsafe { std::mem::transmute(publish) };
        let state: u32 = if on { 2 } else { 1 };
        // SAFETY: state outlives the call and its size is passed.
        let status = unsafe {
            publish(
                ENERGY_SAVER_STATE,
                std::ptr::null(),
                &state,
                4,
                std::ptr::null(),
            )
        };
        if status < 0 {
            return Err(format!(
                "Switching battery saver failed (NTSTATUS {status:#x})."
            ));
        }
        Ok(())
    }
}

#[cfg(not(windows))]
mod win {
    use super::BatterySaverStatus;

    const UNSUPPORTED: &str = "Battery saver control currently requires Windows.";

    pub fn read() -> Result<BatterySaverStatus, String> {
        Err(UNSUPPORTED.into())
    }

    pub fn write_threshold(_threshold: u32) -> Result<(), String> {
        Err(UNSUPPORTED.into())
    }

    pub fn set(_on: bool) -> Result<(), String> {
        Err(UNSUPPORTED.into())
    }
}

fn read() -> BatterySaverStatus {
    win::read().unwrap_or_else(|error| BatterySaverStatus {
        on: None,
        threshold: None,
        supported: false,
        error: Some(error),
    })
}

#[command]
pub async fn get_battery_saver() -> Result<BatterySaverStatus, String> {
    tauri::async_runtime::spawn_blocking(read)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn set_battery_saver_threshold(threshold: u32) -> Result<BatterySaverStatus, String> {
    crate::config_gate::require()?;
    let threshold = validate(threshold)?;
    tauri::async_runtime::spawn_blocking(move || {
        win::write_threshold(threshold)?;
        Ok(read())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[command]
pub async fn set_battery_saver(on: bool) -> Result<BatterySaverStatus, String> {
    crate::config_gate::require()?;
    tauri::async_runtime::spawn_blocking(move || {
        win::set(on)?;
        Ok(read())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threshold_validation() {
        for ok in [0, 20, 50, 100] {
            assert_eq!(validate(ok), Ok(ok));
        }
        for bad in [10, 25, 110] {
            assert!(validate(bad).is_err());
        }
    }

    #[test]
    #[cfg(windows)]
    #[ignore = "read-only battery saver probe"]
    fn battery_saver_live() {
        let s = win::read().unwrap();
        println!("{:?} {:?}", s.on, s.threshold);
    }
}
