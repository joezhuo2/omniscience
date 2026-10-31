//! Offered updates from Windows Update, including optional drivers.
//! Does not inspect arbitrary third-party applications or install anything.
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::State;

#[derive(Clone, Serialize)]
pub struct OfferedUpdate {
    id: String,
    title: String,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScanPhase {
    #[default]
    Idle,
    Checking,
    Complete,
    Failed,
}

#[derive(Clone, Default, Serialize)]
pub struct ScanStatus {
    phase: ScanPhase,
    updates: Vec<OfferedUpdate>,
    error: Option<String>,
}

/// One shared job, independent of IPC request lifetime and dashboard mounting.
#[derive(Default)]
pub struct UpdateChecker {
    state: Mutex<ScanStatus>,
}

impl UpdateChecker {
    pub fn status(&self) -> ScanStatus {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    fn begin(&self) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.phase == ScanPhase::Checking {
            return false;
        }
        *state = ScanStatus {
            phase: ScanPhase::Checking,
            ..ScanStatus::default()
        };
        true
    }

    fn finish(&self, result: Result<Vec<OfferedUpdate>, String>) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        *state = match result {
            Ok(updates) => ScanStatus {
                phase: ScanPhase::Complete,
                updates,
                error: None,
            },
            Err(error) => ScanStatus {
                phase: ScanPhase::Failed,
                updates: vec![],
                error: Some(error),
            },
        };
    }

    pub fn start(self: &Arc<Self>) -> ScanStatus {
        if self.begin() {
            let job = self.clone();
            tauri::async_runtime::spawn(async move {
                let result = tauri::async_runtime::spawn_blocking(scan)
                    .await
                    .map_err(|e| e.to_string())
                    .and_then(|r| r);
                job.finish(result);
            });
        }
        self.status()
    }
}

#[tauri::command]
pub fn scan_driver_updates(checker: State<'_, Arc<UpdateChecker>>) -> ScanStatus {
    checker.start()
}

#[tauri::command]
pub fn get_system_update_status(checker: State<'_, Arc<UpdateChecker>>) -> ScanStatus {
    checker.status()
}

#[cfg(windows)]
fn scan() -> Result<Vec<OfferedUpdate>, String> {
    use serde::Deserialize;
    use std::sync::Mutex;
    static SCAN: Mutex<()> = Mutex::new(());
    let _guard = SCAN
        .try_lock()
        .map_err(|_| "An update check is already running")?;
    // A partial/aborted search must never look up to date.
    let json = crate::system_command::powershell(
        r#"
$session = New-Object -ComObject Microsoft.Update.Session
$session.ClientApplicationID = 'Omniscience'
$result = $session.CreateUpdateSearcher().Search("IsInstalled=0 and IsHidden=0")
if ($result.ResultCode -ne 2) { throw 'Windows Update did not complete the check. Try again.' }
$items = @($result.Updates | ForEach-Object {
    @{ id = $_.Identity.UpdateID; title = $_.Title }
})
ConvertTo-Json -InputObject $items -Compress
"#,
        std::time::Duration::from_secs(180),
    )?;
    #[derive(Deserialize)]
    struct Item {
        id: String,
        title: String,
    }
    let items: Vec<Item> = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    if items.iter().any(|i| !valid_update_id(&i.id)) {
        return Err("Windows Update returned an invalid update identifier. Try again.".into());
    }
    Ok(items
        .into_iter()
        .map(|i| OfferedUpdate {
            id: i.id,
            title: i.title,
        })
        .collect())
}

#[cfg(not(windows))]
fn scan() -> Result<Vec<OfferedUpdate>, String> {
    Err("Driver and software update checks currently require Windows.".into())
}

fn valid_update_id(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
}

#[tauri::command]
pub fn open_update_download(id: String) -> Result<(), String> {
    if !valid_update_id(&id) {
        return Err("Invalid update identifier".into());
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::Shell::ShellExecuteW;
        let url: Vec<u16> = format!("https://www.catalog.update.microsoft.com/Search.aspx?q={id}")
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let verb: Vec<u16> = "open".encode_utf16().chain(Some(0)).collect();
        // SAFETY: both strings are terminated UTF-16 and remain live for the call.
        let result = unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                verb.as_ptr(),
                url.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                1,
            )
        };
        if result as isize <= 32 {
            return Err("Could not open the Microsoft Update Catalog in your browser".into());
        }
        Ok(())
    }
    #[cfg(not(windows))]
    Err("Microsoft Update Catalog downloads currently require Windows.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_background_job_retains_its_result_until_rechecked() {
        let checker = UpdateChecker::default();
        assert!(checker.begin());
        assert!(!checker.begin());
        assert!(checker.status().phase == ScanPhase::Checking);
        checker.finish(Ok(vec![OfferedUpdate {
            id: "id".into(),
            title: "Driver".into(),
        }]));
        let reopened = checker.status();
        assert!(reopened.phase == ScanPhase::Complete);
        assert_eq!(reopened.updates.len(), 1);
        assert!(checker.begin());
        assert!(checker.status().updates.is_empty());
        checker.finish(Err("offline".into()));
        let failed = checker.status();
        assert!(failed.phase == ScanPhase::Failed);
        assert_eq!(failed.error.as_deref(), Some("offline"));
    }

    #[test]
    #[cfg(windows)]
    #[ignore = "contacts the machine's configured Windows Update service"]
    fn update_scan_live() {
        let updates = scan().expect("Windows Update scan");
        assert!(
            updates
                .iter()
                .all(|u| valid_update_id(&u.id) && !u.title.is_empty())
        );
    }

    #[test]
    fn accepts_only_catalog_update_guids() {
        assert!(valid_update_id("12345678-abcd-1234-5678-1234567890ab"));
        for id in [
            "",
            "https://example.com",
            "12345678-abcd-1234-5678-1234567890ab&x=1",
            "zzzzzzzz-abcd-1234-5678-1234567890ab",
        ] {
            assert!(!valid_update_id(id));
        }
    }
}
