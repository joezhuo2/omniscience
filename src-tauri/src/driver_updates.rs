//! Offered updates from Windows Update, including optional drivers.
//! Does not inspect arbitrary third-party applications or install anything.
use serde::Serialize;

#[derive(Serialize)]
pub struct OfferedUpdate {
    id: String,
    title: String,
}

#[tauri::command]
pub async fn scan_driver_updates() -> Result<Vec<OfferedUpdate>, String> {
    tauri::async_runtime::spawn_blocking(scan)
        .await
        .map_err(|e| e.to_string())?
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
