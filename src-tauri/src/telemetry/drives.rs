use std::collections::HashMap;

use sysinfo::{DiskRefreshKind, Disks};

use super::types::DriveInfo;

fn drive_name(mount: &str) -> String {
    if mount == "/" {
        return mount.to_string();
    }
    mount.trim_end_matches(['\\', '/']).to_string()
}

fn wanted(mount: &str) -> bool {
    !(mount.starts_with("/System/Volumes/") || mount.starts_with("/private/"))
}

pub fn list(rates: &HashMap<String, (f64, f64)>) -> Vec<DriveInfo> {
    // Space only: disk kind and I/O stats open every physical drive.
    let mut out: Vec<DriveInfo> =
        Disks::new_with_refreshed_list_specifics(DiskRefreshKind::nothing().with_storage())
            .iter()
            .filter(|d| d.total_space() > 0)
            .filter_map(|d| {
                let mount = d.mount_point().to_string_lossy();
                wanted(&mount).then(|| {
                    let name = drive_name(&mount);
                    let rate = rates.get(&name.to_uppercase());
                    DriveInfo {
                        label: Some(d.name().to_string_lossy().into_owned())
                            .filter(|l| !l.is_empty()),
                        total_bytes: d.total_space(),
                        free_bytes: d.available_space(),
                        read_bps: rate.map(|r| r.0.round()),
                        write_bps: rate.map(|r| r.1.round()),
                        name,
                    }
                })
            })
            .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_drives_by_mount_point() {
        assert_eq!(drive_name(r"C:\"), "C:");
        assert_eq!(drive_name("/"), "/");
        assert_eq!(drive_name("/Volumes/Backup/"), "/Volumes/Backup");
    }

    #[test]
    fn hides_macos_system_volumes() {
        assert!(wanted("/"));
        assert!(wanted("/Volumes/Backup"));
        assert!(!wanted("/System/Volumes/VM"));
        assert!(!wanted("/private/var/vm"));
    }

    #[test]
    fn lists_sorted_nonempty_volumes() {
        let l = list(&HashMap::new());
        assert!(l.iter().all(|d| d.total_bytes > 0));
        assert!(l.windows(2).all(|w| w[0].name <= w[1].name));
    }
}
