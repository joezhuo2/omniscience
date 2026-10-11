//! Process grouping and the guarded process killer.

use std::collections::{HashMap, HashSet};

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

use super::types::{KillOutcome, KillReport, KillResult, ProcessGroup};

/// Processes that must never be killed (OS-critical on Windows and macOS). Compared after
/// [`normalize`], so case and a trailing `.exe` do not matter.
const BLOCKLIST: &[&str] = &[
    // Windows
    "system",
    "idle",
    "system idle process",
    "registry",
    "smss",
    "csrss",
    "wininit",
    "winlogon",
    "services",
    "lsass",
    "lsaiso",
    "svchost",
    "explorer",
    "dwm",
    "fontdrvhost",
    "sihost",
    "ctfmon",
    "memcompression",
    "memory compression",
    "secure system",
    "audiodg",
    "spoolsv",
    "msmpeng",
    // macOS
    "launchd",
    "kernel_task",
    "windowserver",
    "loginwindow",
    "systemuiserver",
    "dock",
    "finder",
    "mds",
    "coreaudiod",
];

/// PIDs that are always refused (System Idle Process and System on Windows; launchd is 1 on
/// macOS but is blocked by name).
const PROTECTED_PIDS: &[u32] = &[0, 4];

/// Group key: lowercase, trimmed, without a trailing `.exe`.
pub fn normalize(name: &str) -> String {
    let lower = name.trim().to_lowercase();
    lower
        .strip_suffix(".exe")
        .map(str::to_string)
        .unwrap_or(lower)
}

pub fn is_protected(key: &str, pid: u32, own_tree: &HashSet<u32>) -> bool {
    PROTECTED_PIDS.contains(&pid)
        || own_tree.contains(&pid)
        || BLOCKLIST.contains(&normalize(key).as_str())
}

/// One process as read from the OS.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcInfo {
    pub pid: u32,
    pub parent: Option<u32>,
    pub name: String,
    /// Percent of one core, as sysinfo reports it.
    pub cpu_core_pct: f32,
    pub mem_bytes: u64,
}

/// `root` and all of its descendants.
pub fn descendants(procs: &[ProcInfo], root: u32) -> HashSet<u32> {
    let mut tree = HashSet::from([root]);
    loop {
        let before = tree.len();
        for p in procs {
            if p.parent.is_some_and(|parent| tree.contains(&parent)) {
                tree.insert(p.pid);
            }
        }
        if tree.len() == before {
            return tree;
        }
    }
}

/// Groups processes by [`normalize`]d name and returns the top `limit` by CPU or memory.
/// The idle pseudo-process is left out (its "CPU usage" is idle time).
pub fn top_groups(
    procs: &[ProcInfo],
    logical_cpus: usize,
    by_memory: bool,
    limit: usize,
) -> Vec<ProcessGroup> {
    let cpus = logical_cpus.max(1) as f32;
    let mut groups: HashMap<String, ProcessGroup> = HashMap::new();
    for p in procs {
        if p.pid == 0 || p.name.is_empty() {
            continue;
        }
        let key = normalize(&p.name);
        if key == "idle" || key == "system idle process" {
            continue;
        }
        let g = groups.entry(key.clone()).or_insert_with(|| ProcessGroup {
            key,
            name: p.name.clone(),
            pids: Vec::new(),
            cpu_pct: 0.0,
            mem_bytes: 0,
        });
        g.pids.push(p.pid);
        g.cpu_pct += p.cpu_core_pct / cpus;
        g.mem_bytes += p.mem_bytes;
    }
    let mut list: Vec<ProcessGroup> = groups.into_values().collect();
    for g in &mut list {
        g.pids.sort_unstable();
        g.cpu_pct = (g.cpu_pct * 10.0).round() / 10.0;
    }
    if by_memory {
        list.sort_by(|a, b| b.mem_bytes.cmp(&a.mem_bytes).then(a.key.cmp(&b.key)));
    } else {
        list.sort_by(|a, b| {
            b.cpu_pct
                .total_cmp(&a.cpu_pct)
                .then(b.mem_bytes.cmp(&a.mem_bytes))
                .then(a.key.cmp(&b.key))
        });
    }
    list.truncate(limit);
    list
}

/// Reads processes from a refreshed `System`.
pub fn read(sys: &System) -> Vec<ProcInfo> {
    sys.processes()
        .values()
        .map(|p| ProcInfo {
            pid: p.pid().as_u32(),
            parent: p.parent().map(|pp| pp.as_u32()),
            name: p.name().to_string_lossy().into_owned(),
            cpu_core_pct: p.cpu_usage(),
            mem_bytes: p.memory(),
        })
        .collect()
}

pub fn refresh(sys: &mut System) {
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_cpu().with_memory(),
    );
}

/// Kills every process whose normalized name is `key`, refusing protected ones.
pub fn kill_group(key: &str) -> KillReport {
    let key = normalize(key);
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    let procs = read(&sys);
    let own = descendants(&procs, std::process::id());
    let results = procs
        .iter()
        .filter(|p| normalize(&p.name) == key)
        .map(|p| {
            if is_protected(&key, p.pid, &own) {
                KillResult {
                    pid: p.pid,
                    outcome: KillOutcome::Protected,
                    message: Some("Protected system or Omniscience process".into()),
                }
            } else {
                kill_pid(p.pid, &sys)
            }
        })
        .collect::<Vec<_>>();
    let results = if results.is_empty() {
        vec![KillResult {
            pid: 0,
            outcome: KillOutcome::NotFound,
            message: Some(format!("No running process named {key}")),
        }]
    } else {
        results
    };
    KillReport { key, results }
}

#[cfg(windows)]
fn kill_pid(pid: u32, _sys: &System) -> KillResult {
    use windows_sys::Win32::Foundation::{
        CloseHandle, ERROR_ACCESS_DENIED, ERROR_INVALID_PARAMETER, GetLastError,
    };
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess};
    let fail = |code: u32| {
        let (outcome, message) = match code {
            ERROR_ACCESS_DENIED => (
                KillOutcome::AccessDenied,
                "Access denied (needs administrator rights)".to_string(),
            ),
            ERROR_INVALID_PARAMETER => (KillOutcome::NotFound, "Process already exited".into()),
            other => (KillOutcome::Failed, format!("Windows error {other}")),
        };
        KillResult {
            pid,
            outcome,
            message: Some(message),
        }
    };
    // SAFETY: the handle is checked for null and closed on every path.
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle.is_null() {
            return fail(GetLastError());
        }
        let ok = TerminateProcess(handle, 1) != 0;
        let err = GetLastError();
        CloseHandle(handle);
        if ok {
            KillResult {
                pid,
                outcome: KillOutcome::Killed,
                message: None,
            }
        } else {
            fail(err)
        }
    }
}

#[cfg(not(windows))]
fn kill_pid(pid: u32, sys: &System) -> KillResult {
    let outcome = match sys.process(sysinfo::Pid::from_u32(pid)) {
        None => KillOutcome::NotFound,
        Some(p) if p.kill() => KillOutcome::Killed,
        Some(_) => KillOutcome::Failed,
    };
    KillResult {
        pid,
        outcome,
        message: (outcome == KillOutcome::Failed).then(|| "Could not end the process".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(pid: u32, parent: Option<u32>, name: &str, cpu: f32, mem: u64) -> ProcInfo {
        ProcInfo {
            pid,
            parent,
            name: name.into(),
            cpu_core_pct: cpu,
            mem_bytes: mem,
        }
    }

    #[test]
    fn normalizes_names() {
        assert_eq!(normalize("Chrome.EXE"), "chrome");
        assert_eq!(normalize(" explorer.exe "), "explorer");
        assert_eq!(normalize("kernel_task"), "kernel_task");
    }

    #[test]
    fn blocks_critical_names_case_insensitively() {
        let own = HashSet::new();
        for name in [
            "explorer.exe",
            "CSRSS.EXE",
            "csrss",
            "lsass.exe",
            "kernel_task",
            "launchd",
        ] {
            assert!(is_protected(name, 1234, &own), "{name}");
        }
        assert!(!is_protected("chrome.exe", 1234, &own));
    }

    #[test]
    fn blocks_system_pids_and_own_tree() {
        let own = HashSet::from([500, 501]);
        assert!(is_protected("anything", 0, &own));
        assert!(is_protected("anything", 4, &own));
        assert!(is_protected("msedgewebview2", 501, &own));
        assert!(!is_protected("msedgewebview2", 900, &own));
    }

    #[test]
    fn finds_descendants() {
        let procs = [
            p(10, None, "omniscience.exe", 0.0, 0),
            p(11, Some(10), "msedgewebview2.exe", 0.0, 0),
            p(12, Some(11), "msedgewebview2.exe", 0.0, 0),
            p(13, Some(1), "chrome.exe", 0.0, 0),
        ];
        assert_eq!(descendants(&procs, 10), HashSet::from([10, 11, 12]));
    }

    #[test]
    fn groups_and_sorts_by_cpu() {
        let procs = [
            p(1, None, "chrome.exe", 40.0, 100),
            p(2, None, "chrome.exe", 8.0, 200),
            p(3, None, "code.exe", 24.0, 900),
            p(0, None, "System Idle Process", 2000.0, 0),
        ];
        let top = top_groups(&procs, 8, false, 8);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].key, "chrome");
        assert_eq!(top[0].pids, vec![1, 2]);
        assert_eq!(top[0].cpu_pct, 6.0);
        assert_eq!(top[0].mem_bytes, 300);
        assert_eq!(top[1].key, "code");
    }

    #[test]
    fn sorts_by_memory_and_truncates() {
        let procs = [
            p(1, None, "a.exe", 90.0, 10),
            p(2, None, "b.exe", 1.0, 30),
            p(3, None, "c.exe", 1.0, 20),
        ];
        let top = top_groups(&procs, 1, true, 2);
        assert_eq!(
            top.iter().map(|g| g.key.as_str()).collect::<Vec<_>>(),
            ["b", "c"]
        );
    }
}
