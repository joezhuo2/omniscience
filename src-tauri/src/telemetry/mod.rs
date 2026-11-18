//! Telemetry engine: a single background thread samples sensors on a fixed interval and emits one
//! [`types::Snapshot`] per tick as the `telemetry://tick` event.

mod alerts;
mod battery;
#[cfg(any(windows, test))]
mod fans;
pub mod history;
#[cfg(any(windows, test))]
mod parse;
mod peak;
mod ping;
#[cfg(windows)]
mod process_io;
pub mod processes;
mod public_ip;
mod sampler;
mod slow;
pub mod types;
mod usage;
#[cfg(windows)]
mod windows;

use std::collections::HashSet;
use std::panic::AssertUnwindSafe;
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

use history::{History, HistorySample};
use types::{Snapshot, Thresholds};

pub const TICK_EVENT: &str = "telemetry://tick";

/// What the collector should do; changed from commands, the tray and the window.
#[derive(Debug, Clone, PartialEq)]
pub struct Control {
    pub expanded: bool,
    pub visible: bool,
    /// Whether the processes section is open, so the process list is sampled.
    pub processes: bool,
    pub interval: Duration,
    pub thresholds: Thresholds,
    /// Host to ping while expanded; empty for none.
    pub ping_target: Arc<str>,
    /// Whether to look up the public IP while expanded.
    pub public_ip: bool,
    pub sustained_alerts: bool,
    pub history_hours: u32,
    pub pill_w: f64,
    pub dodge_tabs: bool,
    pub widget: bool,
}

impl Default for Control {
    fn default() -> Self {
        Self {
            expanded: false,
            visible: true,
            processes: true,
            interval: Duration::from_secs(1),
            thresholds: Thresholds::default(),
            ping_target: "".into(),
            public_ip: false,
            sustained_alerts: false,
            history_hours: 24,
            pill_w: 260.0,
            dodge_tabs: false,
            widget: false,
        }
    }
}

pub struct Telemetry {
    control: Mutex<Control>,
    wake: Condvar,
    latest: Mutex<Option<Snapshot>>,
    history: Mutex<History>,
    usage: Mutex<usage::Usage>,
    process_viewers: Mutex<HashSet<String>>,
}

impl Telemetry {
    pub fn set_process_viewer(&self, label: &str, enabled: bool) {
        let mut viewers = self
            .process_viewers
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if enabled {
            viewers.insert(label.to_owned());
        } else {
            viewers.remove(label);
        }
        let any = !viewers.is_empty();
        self.update(|c| c.processes = any);
    }

    pub fn new(control: Control) -> Arc<Self> {
        Arc::new(Self {
            control: Mutex::new(control),
            wake: Condvar::new(),
            latest: Mutex::new(None),
            history: Mutex::new(History::default()),
            usage: Mutex::new(usage::Usage::open(None)),
            process_viewers: Mutex::new(HashSet::new()),
        })
    }

    pub fn open_history(&self, dir: Option<&Path>) {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or_default();
        let loaded = History::open(dir, now_ms, self.control().history_hours);
        *self.history.lock().unwrap_or_else(|e| e.into_inner()) = loaded;
        *self.usage.lock().unwrap_or_else(|e| e.into_inner()) = usage::Usage::open(dir);
    }

    pub fn history(&self, since_ms: Option<u64>) -> Vec<HistorySample> {
        self.history
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .since(since_ms)
    }

    pub fn latest(&self) -> Option<Snapshot> {
        self.latest
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn control(&self) -> Control {
        self.control
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Applies `f` to the control state and wakes the collector so the change takes effect now.
    pub fn update(&self, f: impl FnOnce(&mut Control)) {
        let mut c = self.control.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut c);
        self.wake.notify_all();
    }

    /// Starts the collector thread. If it panics, it is restarted after a second.
    pub fn start(self: &Arc<Self>, app: AppHandle) {
        let this = Arc::clone(self);
        std::thread::Builder::new()
            .name("telemetry".into())
            .spawn(move || {
                loop {
                    let run = std::panic::catch_unwind(AssertUnwindSafe(|| this.run(&app)));
                    if run.is_ok() {
                        return;
                    }
                    crate::diagnostics::log_error("telemetry collector panicked; restarting");
                    std::thread::sleep(Duration::from_secs(1));
                }
            })
            .expect("spawn telemetry thread");
    }

    fn run(&self, app: &AppHandle) {
        let mut sampler = sampler::Sampler::new();
        let mut sustained = alerts::Sustained::default();
        loop {
            let control = {
                let mut c = self.control.lock().unwrap_or_else(|e| e.into_inner());
                while !c.visible && !c.widget {
                    sampler.suspend_detail();
                    sustained.reset();
                    c = self.wake.wait(c).unwrap_or_else(|e| e.into_inner());
                }
                c.clone()
            };
            sampler.set_network(control.ping_target.clone(), control.public_ip);
            let mut snapshot = sampler.sample(
                control.thresholds,
                control.expanded || control.widget,
                control.processes,
            );
            {
                let mut usage = self.usage.lock().unwrap_or_else(|e| e.into_inner());
                usage.record(snapshot.timestamp_ms, &snapshot.core);
                if let Some(d) = snapshot.detail.as_mut() {
                    let t = usage.totals(snapshot.timestamp_ms);
                    d.data_today_bytes = Some(t.data_today_bytes);
                    d.data_month_bytes = Some(t.data_month_bytes);
                    d.energy_today_wh = t.energy_today_wh;
                }
            }
            if let Err(e) = app.emit(TICK_EVENT, &snapshot) {
                crate::diagnostics::log_error(format_args!("emit telemetry: {e}"));
            }
            if control.sustained_alerts {
                for label in sustained.check(snapshot.timestamp_ms, &snapshot.core) {
                    let shown = app
                        .notification()
                        .builder()
                        .title("High sustained usage")
                        .body(format!("{label} has been above 90% for 30 seconds."))
                        .show();
                    if let Err(e) = shown {
                        crate::diagnostics::log_error(format_args!("notify: {e}"));
                    }
                }
            } else {
                sustained.reset();
            }
            self.history
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .record(
                    snapshot.timestamp_ms,
                    &snapshot.core,
                    battery::percent(),
                    control.history_hours,
                );
            *self.latest.lock().unwrap_or_else(|e| e.into_inner()) = Some(snapshot);
            let c = self.control.lock().unwrap_or_else(|e| e.into_inner());
            // Woken early by any control change (e.g. expand), otherwise once per interval.
            let _ = self
                .wake
                .wait_timeout(c, control.interval)
                .unwrap_or_else(|e| e.into_inner());
        }
    }
}
