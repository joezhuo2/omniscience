//! CPU boost mode stored in the active Windows power plan (PERFBOOSTMODE).
use serde::Serialize;
use tauri::command;

#[derive(Serialize)]
pub struct CpuBoostStatus {
    current: Option<u32>,
    label: Option<String>,
    plan: Option<String>,
    mismatch: bool,
    supported: bool,
    error: Option<String>,
}

#[cfg(any(windows, test))]
fn label(value: u32) -> String {
    match value {
        0 => "Disabled",
        1 => "Enabled",
        2 => "Aggressive",
        3 => "Efficient enabled",
        4 => "Efficient aggressive",
        5 => "Aggressive at guaranteed",
        6 => "Efficient aggressive at guaranteed",
        _ => return format!("Unknown ({value})"),
    }
    .into()
}

#[cfg(any(windows, test))]
fn validate(mode: u32) -> Result<u32, String> {
    if mode > 2 {
        return Err("CPU boost mode must be 0 (Off), 1 (On) or 2 (Aggressive).".into());
    }
    Ok(mode)
}

#[cfg(windows)]
mod win {
    use super::*;
    use windows_sys::Win32::Foundation::{ERROR_SUCCESS, LocalFree};
    use windows_sys::Win32::System::Power::{
        PowerGetActiveScheme, PowerReadACValueIndex, PowerReadDCValueIndex, PowerReadFriendlyName,
        PowerSetActiveScheme, PowerWriteACValueIndex, PowerWriteDCValueIndex,
    };
    use windows_sys::core::GUID;

    const SUB_PROCESSOR: GUID = GUID::from_u128(0x54533251_82be_4824_96c1_47b60b740d00);
    const PERFBOOSTMODE: GUID = GUID::from_u128(0xbe337238_0d82_4146_a960_4f3749d470c7);

    fn check(code: u32, what: &str) -> Result<(), String> {
        if code == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(format!("{what} failed (Windows error {code})."))
        }
    }

    struct Scheme(*mut GUID);
    impl Scheme {
        fn active() -> Result<Self, String> {
            let mut guid = std::ptr::null_mut();
            // SAFETY: guid receives a LocalAlloc'd GUID that Scheme frees on drop.
            check(
                unsafe { PowerGetActiveScheme(std::ptr::null_mut(), &mut guid) },
                "Reading the active power plan",
            )?;
            Ok(Self(guid))
        }
    }
    impl Drop for Scheme {
        fn drop(&mut self) {
            // SAFETY: the pointer came from PowerGetActiveScheme and is freed once.
            unsafe { LocalFree(self.0.cast()) };
        }
    }

    fn plan_name(scheme: &Scheme) -> Option<String> {
        let mut size = 0u32;
        // SAFETY: a null buffer asks for the required size.
        unsafe {
            PowerReadFriendlyName(
                std::ptr::null_mut(),
                scheme.0,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null_mut(),
                &mut size,
            )
        };
        if size < 2 {
            return None;
        }
        let mut buf = vec![0u8; size as usize];
        // SAFETY: buf holds `size` writable bytes.
        let code = unsafe {
            PowerReadFriendlyName(
                std::ptr::null_mut(),
                scheme.0,
                std::ptr::null(),
                std::ptr::null(),
                buf.as_mut_ptr(),
                &mut size,
            )
        };
        if code != ERROR_SUCCESS {
            return None;
        }
        let wide: Vec<u16> = buf
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&c| u16::from_le_bytes(c))
            .take_while(|&c| c != 0)
            .collect();
        Some(String::from_utf16_lossy(&wide)).filter(|s| !s.is_empty())
    }

    pub fn read() -> Result<CpuBoostStatus, String> {
        let scheme = Scheme::active()?;
        let (mut ac, mut dc) = (0u32, 0u32);
        // SAFETY: scheme and the GUID constants are valid; outputs are writable u32s.
        unsafe {
            check(
                PowerReadACValueIndex(
                    std::ptr::null_mut(),
                    scheme.0,
                    &SUB_PROCESSOR,
                    &PERFBOOSTMODE,
                    &mut ac,
                ),
                "Reading CPU boost (AC)",
            )?;
            check(
                PowerReadDCValueIndex(
                    std::ptr::null_mut(),
                    scheme.0,
                    &SUB_PROCESSOR,
                    &PERFBOOSTMODE,
                    &mut dc,
                ),
                "Reading CPU boost (battery)",
            )?;
        }
        Ok(CpuBoostStatus {
            current: Some(ac),
            label: Some(label(ac)),
            plan: plan_name(&scheme),
            mismatch: ac != dc,
            supported: true,
            error: None,
        })
    }

    pub fn write(mode: u32) -> Result<(), String> {
        let scheme = Scheme::active()?;
        // SAFETY: scheme and the GUID constants are valid for the duration of the calls.
        unsafe {
            check(
                PowerWriteACValueIndex(
                    std::ptr::null_mut(),
                    scheme.0,
                    &SUB_PROCESSOR,
                    &PERFBOOSTMODE,
                    mode,
                ),
                "Writing CPU boost (AC)",
            )?;
            check(
                PowerWriteDCValueIndex(
                    std::ptr::null_mut(),
                    scheme.0,
                    &SUB_PROCESSOR,
                    &PERFBOOSTMODE,
                    mode,
                ),
                "Writing CPU boost (battery)",
            )?;
            check(
                PowerSetActiveScheme(std::ptr::null_mut(), scheme.0),
                "Applying the power plan",
            )
        }
    }
}

#[cfg(not(windows))]
const UNSUPPORTED: &str = "CPU boost control currently requires Windows.";

fn read() -> CpuBoostStatus {
    #[cfg(windows)]
    let result = win::read();
    #[cfg(not(windows))]
    let result = Err(UNSUPPORTED.to_string());
    result.unwrap_or_else(|error| CpuBoostStatus {
        current: None,
        label: None,
        plan: None,
        mismatch: false,
        supported: false,
        error: Some(error),
    })
}

#[command]
pub async fn get_cpu_boost() -> Result<CpuBoostStatus, String> {
    tauri::async_runtime::spawn_blocking(read)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn set_cpu_boost(mode: u32) -> Result<CpuBoostStatus, String> {
    let mode = validate(mode)?;
    tauri::async_runtime::spawn_blocking(move || {
        #[cfg(windows)]
        win::write(mode)?;
        #[cfg(not(windows))]
        return Err(UNSUPPORTED.to_string());
        #[cfg(windows)]
        Ok(read())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_and_validation() {
        assert_eq!(label(0), "Disabled");
        assert_eq!(label(3), "Efficient enabled");
        assert_eq!(label(9), "Unknown (9)");
        assert_eq!(validate(2), Ok(2));
        assert!(validate(3).is_err());
    }

    #[test]
    #[cfg(windows)]
    #[ignore = "read-only power plan probe"]
    fn cpu_boost_live() {
        let s = win::read().unwrap();
        println!("{:?} {:?}", s.label, s.plan);
    }
}
