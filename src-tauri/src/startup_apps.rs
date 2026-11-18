use crate::telemetry::types::StartupApp;

#[cfg(windows)]
mod imp {
    use super::StartupApp;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    use windows_sys::Win32::Foundation::ERROR_MORE_DATA;
    use windows_sys::Win32::System::Registry::*;

    const APPROVED: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved";
    const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const RUN_WOW: &str = r"Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Run";

    pub enum Origin {
        Key(String),
        Dir(PathBuf),
    }

    pub struct Loc {
        pub prefix: &'static str,
        pub source: &'static str,
        pub toggle: bool,
        pub hive: HKEY,
        pub approved: String,
        pub origin: Origin,
    }

    struct Key(HKEY);

    impl Drop for Key {
        fn drop(&mut self) {
            // SAFETY: the handle was opened by us and is closed exactly once.
            unsafe { RegCloseKey(self.0) };
        }
    }

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }

    pub fn is_enabled(state: Option<u8>) -> bool {
        state.is_none_or(|b| b % 2 == 0)
    }

    pub fn approved_value(enabled: bool, filetime: u64) -> [u8; 12] {
        let mut v = [0u8; 12];
        if enabled {
            v[0] = 2;
        } else {
            v[0] = 3;
            v[4..].copy_from_slice(&filetime.to_le_bytes());
        }
        v
    }

    pub fn make_id(prefix: &str, name: &str) -> String {
        format!("{prefix}:{name}")
    }

    pub fn parse_id(id: &str) -> Option<(&str, &str)> {
        id.split_once(':')
    }

    pub fn sort_apps(apps: &mut [StartupApp]) {
        apps.sort_by_cached_key(|a| (a.name.to_lowercase(), a.id.clone()));
    }

    fn now_filetime() -> u64 {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        (secs + 11_644_473_600) * 10_000_000
    }

    fn open(hive: HKEY, path: &str, sam: u32) -> Option<Key> {
        let mut h: HKEY = std::ptr::null_mut();
        // SAFETY: path is NUL-terminated and h is a valid out pointer.
        let rc = unsafe { RegOpenKeyExW(hive, wide(path).as_ptr(), 0, sam, &mut h) };
        (rc == 0).then_some(Key(h))
    }

    fn enum_values(hive: HKEY, path: &str) -> Vec<(String, u32, Vec<u8>)> {
        let Some(key) = open(hive, path, KEY_READ) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        let mut name = vec![0u16; 16384];
        let mut data = vec![0u8; 4096];
        let mut index = 0;
        loop {
            let mut nlen = name.len() as u32;
            let mut dlen = data.len() as u32;
            let mut ty = 0u32;
            // SAFETY: buffers and their lengths describe valid writable memory.
            let rc = unsafe {
                RegEnumValueW(
                    key.0,
                    index,
                    name.as_mut_ptr(),
                    &mut nlen,
                    std::ptr::null_mut(),
                    &mut ty,
                    data.as_mut_ptr(),
                    &mut dlen,
                )
            };
            if rc == ERROR_MORE_DATA {
                data.resize(dlen as usize, 0);
                continue;
            }
            if rc != 0 {
                break;
            }
            let n = String::from_utf16_lossy(&name[..nlen as usize]);
            out.push((n, ty, data[..dlen as usize].to_vec()));
            index += 1;
        }
        out
    }

    pub fn set_value(
        hive: HKEY,
        path: &str,
        name: &str,
        ty: u32,
        data: &[u8],
    ) -> Result<(), String> {
        let mut h: HKEY = std::ptr::null_mut();
        // SAFETY: path is NUL-terminated and h is a valid out pointer.
        let rc = unsafe {
            RegCreateKeyExW(
                hive,
                wide(path).as_ptr(),
                0,
                std::ptr::null(),
                0,
                KEY_WRITE,
                std::ptr::null(),
                &mut h,
                std::ptr::null_mut(),
            )
        };
        if rc != 0 {
            return Err(format!("Could not open the startup settings (error {rc})."));
        }
        let key = Key(h);
        // SAFETY: data is a valid buffer of the given length.
        let rc = unsafe {
            RegSetValueExW(
                key.0,
                wide(name).as_ptr(),
                0,
                ty,
                data.as_ptr(),
                data.len() as u32,
            )
        };
        if rc != 0 {
            return Err(format!(
                "Could not change the startup setting (error {rc})."
            ));
        }
        Ok(())
    }

    fn utf16_string(data: &[u8]) -> String {
        let units: Vec<u16> = data
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_le_bytes(*c))
            .take_while(|&u| u != 0)
            .collect();
        String::from_utf16_lossy(&units)
    }

    fn entries(loc: &Loc) -> Vec<(String, String, String)> {
        match &loc.origin {
            Origin::Key(path) => enum_values(loc.hive, path)
                .into_iter()
                .filter(|(_, ty, _)| *ty == REG_SZ || *ty == REG_EXPAND_SZ)
                .map(|(n, _, d)| (n.clone(), n, utf16_string(&d)))
                .collect(),
            Origin::Dir(dir) => std::fs::read_dir(dir)
                .into_iter()
                .flatten()
                .flatten()
                .filter(|e| e.path().is_file())
                .filter_map(|e| {
                    let file = e.file_name().to_string_lossy().into_owned();
                    if file.eq_ignore_ascii_case("desktop.ini") {
                        return None;
                    }
                    let stem = e.path().file_stem()?.to_string_lossy().into_owned();
                    Some((file, stem, e.path().to_string_lossy().into_owned()))
                })
                .collect(),
        }
    }

    pub fn list_from(locs: &[Loc]) -> Vec<StartupApp> {
        let mut apps = Vec::new();
        for loc in locs {
            let states = enum_values(loc.hive, &loc.approved);
            for (key, name, command) in entries(loc) {
                let state = states
                    .iter()
                    .find(|(n, ty, _)| *ty == REG_BINARY && n.eq_ignore_ascii_case(&key))
                    .and_then(|(_, _, d)| d.first().copied());
                apps.push(StartupApp {
                    id: make_id(loc.prefix, &key),
                    name,
                    command,
                    source: loc.source.into(),
                    enabled: is_enabled(state),
                    can_toggle: loc.toggle,
                });
            }
        }
        sort_apps(&mut apps);
        apps
    }

    pub fn set_from(locs: &[Loc], id: &str, enabled: bool) -> Result<(), String> {
        let (prefix, key) = parse_id(id).ok_or("Unknown startup app.")?;
        let loc = locs
            .iter()
            .find(|l| l.prefix == prefix)
            .ok_or("Unknown startup app.")?;
        if !entries(loc).iter().any(|(k, _, _)| k == key) {
            return Err("Unknown startup app.".into());
        }
        if !loc.toggle {
            return Err("Changing this startup app needs administrator rights.".into());
        }
        set_value(
            loc.hive,
            &loc.approved,
            key,
            REG_BINARY,
            &approved_value(enabled, now_filetime()),
        )
    }

    pub fn real_locs() -> Vec<Loc> {
        let approved = |s: &str| format!(r"{APPROVED}\{s}");
        let mut locs = vec![
            Loc {
                prefix: "user",
                source: "user",
                toggle: true,
                hive: HKEY_CURRENT_USER,
                approved: approved("Run"),
                origin: Origin::Key(RUN.into()),
            },
            Loc {
                prefix: "machine",
                source: "machine",
                toggle: false,
                hive: HKEY_LOCAL_MACHINE,
                approved: approved("Run"),
                origin: Origin::Key(RUN.into()),
            },
            Loc {
                prefix: "machine32",
                source: "machine",
                toggle: false,
                hive: HKEY_LOCAL_MACHINE,
                approved: approved("Run32"),
                origin: Origin::Key(RUN_WOW.into()),
            },
        ];
        if let Some(appdata) = std::env::var_os("APPDATA") {
            locs.push(Loc {
                prefix: "folder",
                source: "folder",
                toggle: true,
                hive: HKEY_CURRENT_USER,
                approved: approved("StartupFolder"),
                origin: Origin::Dir(
                    PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs\Startup"),
                ),
            });
        }
        locs
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn app(id: &str, name: &str) -> StartupApp {
            StartupApp {
                id: id.into(),
                name: name.into(),
                command: String::new(),
                source: "user".into(),
                enabled: true,
                can_toggle: true,
            }
        }

        #[test]
        fn state_byte_decoding() {
            assert!(is_enabled(None));
            assert!(is_enabled(Some(2)));
            assert!(is_enabled(Some(6)));
            assert!(!is_enabled(Some(3)));
        }

        #[test]
        fn approved_bytes() {
            assert_eq!(
                approved_value(true, 123),
                [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
            );
            let v = approved_value(false, 0x0102030405060708);
            assert_eq!(v[..4], [3, 0, 0, 0]);
            assert_eq!(v[4..], 0x0102030405060708u64.to_le_bytes());
        }

        #[test]
        fn id_round_trip() {
            let id = make_id("user", "My: App");
            assert_eq!(parse_id(&id), Some(("user", "My: App")));
            assert_eq!(parse_id("nocolon"), None);
        }

        #[test]
        fn sorts_case_insensitively() {
            let mut v = vec![app("c", "beta"), app("a", "Zed"), app("b", "Alpha")];
            sort_apps(&mut v);
            let names: Vec<_> = v.iter().map(|a| a.name.as_str()).collect();
            assert_eq!(names, ["Alpha", "beta", "Zed"]);
        }

        fn sz(s: &str) -> Vec<u8> {
            wide(s).iter().flat_map(|u| u.to_le_bytes()).collect()
        }

        #[test]
        fn registry_round_trip_on_test_key() {
            let base = r"Software\OmniscienceTest";
            let run = format!(r"{base}\Run");
            let loc = |toggle| Loc {
                prefix: "user",
                source: "user",
                toggle,
                hive: HKEY_CURRENT_USER,
                approved: format!(r"{base}\Approved"),
                origin: Origin::Key(run.clone()),
            };
            set_value(HKEY_CURRENT_USER, &run, "Beta", REG_SZ, &sz(r"C:\b.exe")).unwrap();
            set_value(HKEY_CURRENT_USER, &run, "alpha", REG_SZ, &sz(r"C:\a.exe")).unwrap();

            let locs = [loc(true)];
            let apps = list_from(&locs);
            assert_eq!(apps.len(), 2);
            assert_eq!(apps[0].name, "alpha");
            assert_eq!(apps[0].command, r"C:\a.exe");
            assert!(apps.iter().all(|a| a.enabled && a.can_toggle));

            set_from(&locs, "user:Beta", false).unwrap();
            let apps = list_from(&locs);
            assert!(apps[0].enabled);
            assert!(!apps[1].enabled);

            set_from(&locs, "user:Beta", true).unwrap();
            assert!(list_from(&locs).iter().all(|a| a.enabled));

            assert!(set_from(&locs, "user:Missing", true).is_err());
            assert!(set_from(&locs, "bogus:Beta", true).is_err());
            assert!(set_from(&[loc(false)], "user:Beta", false).is_err());

            // SAFETY: path is NUL-terminated.
            unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, wide(base).as_ptr()) };
        }

        #[test]
        #[ignore]
        fn live_read_only() {
            for a in list_from(&real_locs()) {
                println!("{} [{}] enabled={}", a.name, a.source, a.enabled);
            }
        }
    }
}

fn list() -> Vec<StartupApp> {
    #[cfg(windows)]
    return imp::list_from(&imp::real_locs());
    #[cfg(not(windows))]
    Vec::new()
}

fn set_enabled(id: &str, enabled: bool) -> Result<(), String> {
    #[cfg(windows)]
    return imp::set_from(&imp::real_locs(), id, enabled);
    #[cfg(not(windows))]
    {
        let _ = (id, enabled);
        Err("Startup apps are not supported on this device.".into())
    }
}

#[tauri::command]
pub async fn get_startup_apps() -> Result<Vec<StartupApp>, String> {
    tauri::async_runtime::spawn_blocking(list)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_startup_app_enabled(id: String, enabled: bool) -> Result<Vec<StartupApp>, String> {
    crate::config_gate::require()?;
    tauri::async_runtime::spawn_blocking(move || {
        set_enabled(&id, enabled)?;
        Ok(list())
    })
    .await
    .map_err(|e| e.to_string())?
}
