use std::collections::{HashMap, HashSet};

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use super::processes::{ProcInfo, descendants, is_protected, kill_pid, read};
use super::types::{
    ActionOutcome, ActionResult, KillOutcome, KillResult, ProcessDetails, ProcessPriority,
};

fn result(pid: u32, outcome: ActionOutcome, message: Option<String>) -> ActionResult {
    ActionResult {
        pid,
        outcome,
        message,
    }
}

fn protected(pid: u32) -> ActionResult {
    result(
        pid,
        ActionOutcome::Protected,
        Some("Protected system or Omniscience process".into()),
    )
}

fn from_kill(k: KillResult) -> ActionResult {
    let outcome = match k.outcome {
        KillOutcome::Killed => ActionOutcome::Ok,
        KillOutcome::Protected => ActionOutcome::Protected,
        KillOutcome::AccessDenied => ActionOutcome::AccessDenied,
        KillOutcome::NotFound => ActionOutcome::NotFound,
        KillOutcome::Failed => ActionOutcome::Failed,
    };
    result(k.pid, outcome, k.message)
}

fn blocked(procs: &[ProcInfo], pid: u32, own: &HashSet<u32>) -> bool {
    let name = procs
        .iter()
        .find(|p| p.pid == pid)
        .map_or("", |p| p.name.as_str());
    is_protected(name, pid, own)
}

fn tree_plan(procs: &[ProcInfo], root: u32, own: &HashSet<u32>) -> Vec<(u32, bool)> {
    let root_blocked = blocked(procs, root, own);
    if root_blocked || !procs.iter().any(|p| p.pid == root) {
        return vec![(root, root_blocked)];
    }
    let set = descendants(procs, root);
    let parents: HashMap<u32, Option<u32>> = procs.iter().map(|p| (p.pid, p.parent)).collect();
    let depth = |pid: u32| {
        let (mut d, mut cur) = (0, pid);
        while let Some(Some(parent)) = parents.get(&cur) {
            if !set.contains(parent) || d >= set.len() {
                break;
            }
            d += 1;
            cur = *parent;
        }
        d
    };
    let mut order: Vec<u32> = set.iter().copied().collect();
    order.sort_by_key(|&pid| (std::cmp::Reverse(depth(pid)), pid));
    order
        .into_iter()
        .map(|pid| (pid, blocked(procs, pid, own)))
        .collect()
}

pub struct Ctx {
    sys: System,
    procs: Vec<ProcInfo>,
    own: HashSet<u32>,
}

impl Ctx {
    pub fn now() -> Self {
        let mut sys = System::new();
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing(),
        );
        let procs = read(&sys);
        let own = descendants(&procs, std::process::id());
        Self { sys, procs, own }
    }

    fn each(&self, pids: Vec<u32>, act: impl Fn(u32) -> ActionResult) -> Vec<ActionResult> {
        pids.into_iter()
            .map(|pid| {
                if blocked(&self.procs, pid, &self.own) {
                    protected(pid)
                } else {
                    act(pid)
                }
            })
            .collect()
    }

    pub fn set_priority(&self, pids: Vec<u32>, priority: ProcessPriority) -> Vec<ActionResult> {
        self.each(pids, |pid| os::set_priority(pid, priority))
    }

    pub fn set_efficiency(&self, pids: Vec<u32>, enabled: bool) -> Vec<ActionResult> {
        self.each(pids, |pid| os::set_efficiency(pid, enabled))
    }

    pub fn suspend(&self, pids: Vec<u32>) -> Vec<ActionResult> {
        self.each(pids, os::suspend)
    }

    pub fn resume(&self, pids: Vec<u32>) -> Vec<ActionResult> {
        self.each(pids, os::resume)
    }

    pub fn kill_tree(&self, pids: Vec<u32>) -> Vec<ActionResult> {
        pids.into_iter()
            .flat_map(|root| tree_plan(&self.procs, root, &self.own))
            .map(|(pid, blocked)| {
                if blocked {
                    protected(pid)
                } else {
                    from_kill(kill_pid(pid, &self.sys))
                }
            })
            .collect()
    }
}

pub fn details(pid: u32) -> Result<ProcessDetails, String> {
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
        true,
        ProcessRefreshKind::nothing()
            .with_exe(UpdateKind::Always)
            .with_cmd(UpdateKind::Always),
    );
    let p = sys
        .process(Pid::from_u32(pid))
        .ok_or_else(|| format!("No running process with id {pid}"))?;
    let command_line = (!p.cmd().is_empty()).then(|| {
        p.cmd()
            .iter()
            .map(|a| a.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
    });
    let (priority, efficiency, suspended) = os::state(pid);
    Ok(ProcessDetails {
        pid,
        name: p.name().to_string_lossy().into_owned(),
        path: p.exe().map(|e| e.display().to_string()),
        command_line,
        priority,
        efficiency,
        suspended,
    })
}

pub fn open_location(pid: u32) -> Result<(), String> {
    let path = details(pid)?
        .path
        .ok_or("The executable path is not available")?;
    #[cfg(windows)]
    let spawned = {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("explorer.exe")
            .raw_arg(format!("/select,\"{path}\""))
            .spawn()
    };
    #[cfg(not(windows))]
    let spawned = std::process::Command::new("open")
        .arg("-R")
        .arg(&path)
        .spawn();
    spawned.map(|_| ()).map_err(|e| e.to_string())
}

#[cfg(windows)]
mod os {
    use std::ffi::c_void;
    use std::mem::{size_of, transmute, zeroed};
    use std::ptr::null_mut;

    use windows_sys::Win32::Foundation::{
        CloseHandle, ERROR_ACCESS_DENIED, ERROR_INVALID_PARAMETER, GetLastError, HANDLE,
        INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
    };
    use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
    use windows_sys::Win32::System::Threading::{
        ABOVE_NORMAL_PRIORITY_CLASS, BELOW_NORMAL_PRIORITY_CLASS, GetPriorityClass,
        GetProcessInformation, HIGH_PRIORITY_CLASS, IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS,
        OpenProcess, OpenThread, PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        PROCESS_POWER_THROTTLING_EXECUTION_SPEED, PROCESS_POWER_THROTTLING_STATE,
        PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION, PROCESS_SUSPEND_RESUME,
        ProcessPowerThrottling, SetPriorityClass, SetProcessInformation,
        THREAD_QUERY_LIMITED_INFORMATION,
    };

    use super::{ActionOutcome, ActionResult, ProcessPriority, result};

    type Failure = (ActionOutcome, String);
    type NtProcess = unsafe extern "system" fn(HANDLE) -> i32;
    type NtQueryThread = unsafe extern "system" fn(HANDLE, u32, *mut c_void, u32, *mut u32) -> i32;

    const THREAD_SUSPEND_COUNT: u32 = 35;
    const STATUS_ACCESS_DENIED: i32 = 0xC000_0022_u32 as i32;
    const STATUS_INVALID_CID: i32 = 0xC000_000B_u32 as i32;

    struct Handle(HANDLE);

    impl Drop for Handle {
        fn drop(&mut self) {
            // SAFETY: the handle is valid, owned by this wrapper and closed once.
            unsafe { CloseHandle(self.0) };
        }
    }

    pub fn failure(code: u32) -> Failure {
        match code {
            ERROR_ACCESS_DENIED => (
                ActionOutcome::AccessDenied,
                "Access denied (needs administrator rights)".into(),
            ),
            ERROR_INVALID_PARAMETER => (ActionOutcome::NotFound, "Process already exited".into()),
            other => (ActionOutcome::Failed, format!("Windows error {other}")),
        }
    }

    pub fn nt_failure(status: i32) -> Failure {
        match status {
            STATUS_ACCESS_DENIED => failure(ERROR_ACCESS_DENIED),
            STATUS_INVALID_CID => failure(ERROR_INVALID_PARAMETER),
            other => (ActionOutcome::Failed, format!("NTSTATUS {other:#x}")),
        }
    }

    fn last() -> Failure {
        // SAFETY: GetLastError only reads thread-local state.
        failure(unsafe { GetLastError() })
    }

    pub fn to_class(p: ProcessPriority) -> u32 {
        match p {
            ProcessPriority::Idle => IDLE_PRIORITY_CLASS,
            ProcessPriority::BelowNormal => BELOW_NORMAL_PRIORITY_CLASS,
            ProcessPriority::Normal => NORMAL_PRIORITY_CLASS,
            ProcessPriority::AboveNormal => ABOVE_NORMAL_PRIORITY_CLASS,
            ProcessPriority::High => HIGH_PRIORITY_CLASS,
        }
    }

    pub fn from_class(class: u32) -> Option<ProcessPriority> {
        Some(match class {
            IDLE_PRIORITY_CLASS => ProcessPriority::Idle,
            BELOW_NORMAL_PRIORITY_CLASS => ProcessPriority::BelowNormal,
            NORMAL_PRIORITY_CLASS => ProcessPriority::Normal,
            ABOVE_NORMAL_PRIORITY_CLASS => ProcessPriority::AboveNormal,
            HIGH_PRIORITY_CLASS => ProcessPriority::High,
            _ => return None,
        })
    }

    fn open(pid: u32, access: u32) -> Result<Handle, Failure> {
        // SAFETY: the returned handle is checked for null and owned by `Handle`.
        let h = unsafe { OpenProcess(access, 0, pid) };
        if h.is_null() {
            Err(last())
        } else {
            Ok(Handle(h))
        }
    }

    fn done(pid: u32, r: Result<(), Failure>) -> ActionResult {
        match r {
            Ok(()) => result(pid, ActionOutcome::Ok, None),
            Err((outcome, message)) => result(pid, outcome, Some(message)),
        }
    }

    fn set_class(h: &Handle, p: ProcessPriority) -> Result<(), Failure> {
        // SAFETY: `h` is a live process handle with PROCESS_SET_INFORMATION.
        if unsafe { SetPriorityClass(h.0, to_class(p)) } != 0 {
            Ok(())
        } else {
            Err(last())
        }
    }

    fn throttle(h: &Handle, enabled: bool) -> Result<(), Failure> {
        let state = PROCESS_POWER_THROTTLING_STATE {
            Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
            ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
            StateMask: if enabled {
                PROCESS_POWER_THROTTLING_EXECUTION_SPEED
            } else {
                0
            },
        };
        // SAFETY: `state` outlives the call and the size matches its type.
        let ok = unsafe {
            SetProcessInformation(
                h.0,
                ProcessPowerThrottling,
                (&raw const state).cast(),
                size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
            )
        };
        if ok != 0 { Ok(()) } else { Err(last()) }
    }

    pub fn set_priority(pid: u32, p: ProcessPriority) -> ActionResult {
        done(
            pid,
            open(pid, PROCESS_SET_INFORMATION).and_then(|h| set_class(&h, p)),
        )
    }

    pub fn set_efficiency(pid: u32, enabled: bool) -> ActionResult {
        done(
            pid,
            open(pid, PROCESS_SET_INFORMATION).and_then(|h| {
                throttle(&h, enabled)?;
                set_class(
                    &h,
                    if enabled {
                        ProcessPriority::Idle
                    } else {
                        ProcessPriority::Normal
                    },
                )
            }),
        )
    }

    fn ntdll(name: &[u8]) -> Option<unsafe extern "system" fn() -> isize> {
        // SAFETY: both names are NUL-terminated and ntdll is always loaded.
        unsafe {
            GetProcAddress(
                GetModuleHandleA(c"ntdll.dll".as_ptr().cast()),
                name.as_ptr(),
            )
        }
    }

    fn nt_process(pid: u32, name: &[u8]) -> ActionResult {
        done(
            pid,
            open(pid, PROCESS_SUSPEND_RESUME).and_then(|h| {
                let f = ntdll(name).ok_or((
                    ActionOutcome::Failed,
                    "Required system function is unavailable".to_string(),
                ))?;
                // SAFETY: NtSuspendProcess and NtResumeProcess take one handle and return an NTSTATUS.
                let f = unsafe { transmute::<unsafe extern "system" fn() -> isize, NtProcess>(f) };
                // SAFETY: `h` is a live handle with PROCESS_SUSPEND_RESUME.
                let status = unsafe { f(h.0) };
                if status >= 0 {
                    Ok(())
                } else {
                    Err(nt_failure(status))
                }
            }),
        )
    }

    pub fn suspend(pid: u32) -> ActionResult {
        nt_process(pid, b"NtSuspendProcess\0")
    }

    pub fn resume(pid: u32) -> ActionResult {
        nt_process(pid, b"NtResumeProcess\0")
    }

    fn priority(pid: u32) -> Option<ProcessPriority> {
        let h = open(pid, PROCESS_QUERY_LIMITED_INFORMATION).ok()?;
        // SAFETY: `h` is a live process handle.
        from_class(unsafe { GetPriorityClass(h.0) })
    }

    fn efficiency(pid: u32) -> Option<bool> {
        let h = open(pid, PROCESS_QUERY_LIMITED_INFORMATION).ok()?;
        // SAFETY: an all-zero state is valid plain data.
        let mut s: PROCESS_POWER_THROTTLING_STATE = unsafe { zeroed() };
        s.Version = PROCESS_POWER_THROTTLING_CURRENT_VERSION;
        // SAFETY: `s` outlives the call and the size matches its type.
        let ok = unsafe {
            GetProcessInformation(
                h.0,
                ProcessPowerThrottling,
                (&raw mut s).cast(),
                size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
            )
        };
        let bit = PROCESS_POWER_THROTTLING_EXECUTION_SPEED;
        (ok != 0).then_some(s.ControlMask & bit != 0 && s.StateMask & bit != 0)
    }

    fn suspended(pid: u32) -> Option<bool> {
        let query = ntdll(b"NtQueryInformationThread\0")?;
        // SAFETY: NtQueryInformationThread has the NtQueryThread signature.
        let query =
            unsafe { transmute::<unsafe extern "system" fn() -> isize, NtQueryThread>(query) };
        // SAFETY: the snapshot handle is checked and owned by `Handle`.
        let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
        if snap == INVALID_HANDLE_VALUE {
            return None;
        }
        let snap = Handle(snap);
        // SAFETY: an all-zero THREADENTRY32 is valid plain data.
        let mut e: THREADENTRY32 = unsafe { zeroed() };
        e.dwSize = size_of::<THREADENTRY32>() as u32;
        let (mut total, mut stopped) = (0, 0);
        // SAFETY: `snap` is a live snapshot and `e.dwSize` is set.
        let mut more = unsafe { Thread32First(snap.0, &mut e) };
        while more != 0 {
            if e.th32OwnerProcessID == pid {
                // SAFETY: the thread handle is checked and owned by `Handle`.
                let th = unsafe { OpenThread(THREAD_QUERY_LIMITED_INFORMATION, 0, e.th32ThreadID) };
                if th.is_null() {
                    return None;
                }
                let th = Handle(th);
                let mut count = 0u32;
                // SAFETY: `count` is a 4-byte buffer as the class requires.
                let status = unsafe {
                    query(
                        th.0,
                        THREAD_SUSPEND_COUNT,
                        (&raw mut count).cast(),
                        size_of::<u32>() as u32,
                        null_mut(),
                    )
                };
                if status < 0 {
                    return None;
                }
                total += 1;
                stopped += i32::from(count > 0);
            }
            // SAFETY: `snap` is a live snapshot and `e.dwSize` is set.
            more = unsafe { Thread32Next(snap.0, &mut e) };
        }
        (total > 0).then_some(stopped == total)
    }

    pub fn state(pid: u32) -> (Option<ProcessPriority>, Option<bool>, Option<bool>) {
        (priority(pid), efficiency(pid), suspended(pid))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn priority_round_trips_through_os_values() {
            for p in [
                ProcessPriority::Idle,
                ProcessPriority::BelowNormal,
                ProcessPriority::Normal,
                ProcessPriority::AboveNormal,
                ProcessPriority::High,
            ] {
                assert_eq!(from_class(to_class(p)), Some(p));
            }
            assert_eq!(to_class(ProcessPriority::Idle), 0x40);
            assert_eq!(to_class(ProcessPriority::High), 0x80);
            assert_eq!(from_class(0x100), None);
        }

        #[test]
        fn maps_os_errors() {
            assert_eq!(failure(5).0, ActionOutcome::AccessDenied);
            assert_eq!(failure(87).0, ActionOutcome::NotFound);
            assert_eq!(failure(1).0, ActionOutcome::Failed);
            assert_eq!(
                nt_failure(STATUS_ACCESS_DENIED).0,
                ActionOutcome::AccessDenied
            );
            assert_eq!(nt_failure(STATUS_INVALID_CID).0, ActionOutcome::NotFound);
            assert_eq!(nt_failure(-1).0, ActionOutcome::Failed);
        }
    }
}

#[cfg(not(windows))]
mod os {
    use super::{ActionOutcome, ActionResult, ProcessPriority, result};

    fn unsupported(pid: u32) -> ActionResult {
        result(
            pid,
            ActionOutcome::Failed,
            Some("Not supported on this platform".into()),
        )
    }

    pub fn set_priority(pid: u32, _: ProcessPriority) -> ActionResult {
        unsupported(pid)
    }

    pub fn set_efficiency(pid: u32, _: bool) -> ActionResult {
        unsupported(pid)
    }

    pub fn suspend(pid: u32) -> ActionResult {
        unsupported(pid)
    }

    pub fn resume(pid: u32) -> ActionResult {
        unsupported(pid)
    }

    pub fn state(_: u32) -> (Option<ProcessPriority>, Option<bool>, Option<bool>) {
        (None, None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(pid: u32, parent: Option<u32>, name: &str) -> ProcInfo {
        ProcInfo {
            pid,
            parent,
            name: name.into(),
            cpu_core_pct: 0.0,
            mem_bytes: 0,
            gpu_pct: 0.0,
            gpu_mem_bytes: 0,
            disk_bps: None,
            net_bps: None,
        }
    }

    #[test]
    fn orders_children_before_parents_and_flags_protected() {
        let procs = [
            p(10, Some(1), "app.exe"),
            p(11, Some(10), "helper.exe"),
            p(12, Some(11), "helper.exe"),
            p(13, Some(10), "lsass.exe"),
            p(14, Some(10), "own.exe"),
            p(99, Some(1), "other.exe"),
        ];
        let own = HashSet::from([14]);
        assert_eq!(
            tree_plan(&procs, 10, &own),
            vec![
                (12, false),
                (11, false),
                (13, true),
                (14, true),
                (10, false)
            ]
        );
    }

    #[test]
    fn protected_root_keeps_its_whole_tree() {
        let procs = [
            p(20, Some(1), "lsass.exe"),
            p(21, Some(20), "app.exe"),
            p(22, Some(21), "helper.exe"),
        ];
        assert_eq!(tree_plan(&procs, 20, &HashSet::new()), vec![(20, true)]);
    }

    #[test]
    fn unknown_root_is_planned_alone_and_system_pids_are_protected() {
        let own = HashSet::new();
        assert_eq!(tree_plan(&[], 4242, &own), vec![(4242, false)]);
        assert_eq!(tree_plan(&[], 4, &own), vec![(4, true)]);
    }

    #[test]
    fn maps_kill_outcomes() {
        let k = |outcome| KillResult {
            pid: 7,
            outcome,
            message: None,
        };
        assert_eq!(from_kill(k(KillOutcome::Killed)).outcome, ActionOutcome::Ok);
        assert_eq!(
            from_kill(k(KillOutcome::AccessDenied)).outcome,
            ActionOutcome::AccessDenied
        );
        assert_eq!(
            from_kill(k(KillOutcome::NotFound)).outcome,
            ActionOutcome::NotFound
        );
        assert_eq!(
            from_kill(k(KillOutcome::Failed)).outcome,
            ActionOutcome::Failed
        );
    }

    #[test]
    fn unknown_pid_has_no_details() {
        assert!(details(u32::MAX - 1).is_err());
    }

    #[cfg(windows)]
    struct Child(std::process::Child);

    #[cfg(windows)]
    impl Drop for Child {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    #[cfg(windows)]
    #[test]
    fn controls_a_spawned_child_process() {
        let mut child = Child(
            std::process::Command::new("ping")
                .args(["-n", "30", "127.0.0.1"])
                .stdout(std::process::Stdio::null())
                .spawn()
                .expect("spawn ping"),
        );
        let pid = child.0.id();
        let ok = |r: Vec<ActionResult>| {
            assert_eq!(r.len(), 1, "{r:?}");
            assert_eq!(r[0].pid, pid);
            assert_eq!(r[0].outcome, ActionOutcome::Ok, "{r:?}");
        };

        let d = details(pid).unwrap();
        assert!(d.name.to_lowercase().starts_with("ping"));
        assert!(
            d.path
                .is_some_and(|p| p.to_lowercase().ends_with("ping.exe"))
        );
        assert!(d.command_line.is_some_and(|c| c.contains("127.0.0.1")));
        assert_eq!(d.priority, Some(ProcessPriority::Normal));
        assert_eq!(d.efficiency, Some(false));
        assert_eq!(d.suspended, Some(false));

        let guarded = Ctx::now().suspend(vec![pid]);
        assert_eq!(guarded[0].outcome, ActionOutcome::Protected);
        assert_eq!(
            Ctx::now().kill_tree(vec![pid])[0].outcome,
            ActionOutcome::Protected
        );

        let mut ctx = Ctx::now();
        ctx.own.clear();
        ok(ctx.set_priority(vec![pid], ProcessPriority::BelowNormal));
        assert_eq!(
            details(pid).unwrap().priority,
            Some(ProcessPriority::BelowNormal)
        );

        ok(ctx.set_efficiency(vec![pid], true));
        let d = details(pid).unwrap();
        assert_eq!(d.efficiency, Some(true));
        assert_eq!(d.priority, Some(ProcessPriority::Idle));
        ok(ctx.set_efficiency(vec![pid], false));
        let d = details(pid).unwrap();
        assert_eq!(d.efficiency, Some(false));
        assert_eq!(d.priority, Some(ProcessPriority::Normal));

        ok(ctx.suspend(vec![pid]));
        assert_eq!(details(pid).unwrap().suspended, Some(true));
        ok(ctx.resume(vec![pid]));
        assert_eq!(details(pid).unwrap().suspended, Some(false));

        let killed = ctx.kill_tree(vec![pid]);
        assert!(
            killed
                .iter()
                .any(|r| r.pid == pid && r.outcome == ActionOutcome::Ok),
            "{killed:?}"
        );
        assert!(killed.iter().all(|r| r.outcome == ActionOutcome::Ok));
        child.0.wait().unwrap();
    }
}
