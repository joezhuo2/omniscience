//! Telemetry engine: a single background thread samples sensors on a fixed interval and emits one
//! [`types::Snapshot`] per tick as the `telemetry://tick` event.

mod alerts;
mod battery;
mod drives;
mod events;
pub mod export;
#[cfg(any(windows, test))]
mod fans;
pub mod history;
#[cfg(windows)]
mod nvml;
#[cfg(any(windows, test))]
mod parse;
mod peak;
mod ping;
pub mod process_actions;
#[cfg(windows)]
mod process_io;
pub mod processes;
mod public_ip;
mod sampler;
mod slow;
pub mod types;
mod usage;
#[cfg(windows)]
mod wifi;
#[cfg(windows)]
mod windows;

use std::collections::HashSet;
use std::panic::AssertUnwindSafe;
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

use events::{EventLog, Watch};
use history::{History, HistorySample};
use types::{HistoryEvent, HistoryEventKind, Snapshot, Thresholds};

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
    pub alerts: crate::settings::AlertSettings,
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
            alerts: crate::settings::AlertSettings::default(),
        }
    }
}

pub struct Telemetry {
    control: Mutex<Control>,
    wake: Condvar,
    latest: Mutex<Option<Snapshot>>,
    history: Mutex<History>,
    events: Mutex<EventLog>,
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
            events: Mutex::new(EventLog::default()),
            usage: Mutex::new(usage::Usage::open(None)),
            process_viewers: Mutex::new(HashSet::new()),
        })
    }

    pub fn open_history(&self, dir: Option<&Path>) {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or_default();
        let hours = self.control().history_hours;
        let loaded = History::open(dir, now_ms, hours);
        *self.history.lock().unwrap_or_else(|e| e.into_inner()) = loaded;
        *self.events.lock().unwrap_or_else(|e| e.into_inner()) = EventLog::open(dir, now_ms, hours);
        *self.usage.lock().unwrap_or_else(|e| e.into_inner()) = usage::Usage::open(dir);
    }

    pub fn history(&self, since_ms: Option<u64>) -> Vec<HistorySample> {
        self.history
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .since(since_ms)
    }

    pub fn history_events(&self, since_ms: Option<u64>) -> Vec<HistoryEvent> {
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .since(since_ms)
    }

    fn record_events(&self, events: Vec<HistoryEvent>, now_ms: u64, hours: u32) {
        let mut log = self.events.lock().unwrap_or_else(|e| e.into_inner());
        for event in events {
            log.record(event, now_ms, hours);
        }
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
        let mut engine = alerts::Engine::default();
        let mut watch = Watch::default();
        loop {
            let control = {
                let mut c = self.control.lock().unwrap_or_else(|e| e.into_inner());
                while !c.visible && !c.widget {
                    sampler.suspend_detail();
                    engine.reset();
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
            let charge = battery::charge();
            let ts_ms = snapshot.timestamp_ms;
            let mut events = Vec::new();
            let mut push = |kind, label: String| {
                events.push(HistoryEvent { ts_ms, kind, label });
            };
            if let Some(label) = watch.power(charge.map(|c| c.1)) {
                push(HistoryEventKind::PowerSource, label.to_owned());
            }
            if let Some(d) = &snapshot.detail {
                let gpus = d.gpus.iter().map(|g| (g.name.as_str(), g.throttling));
                for label in watch.throttle(gpus, d.cpu_limit_pct) {
                    push(HistoryEventKind::Throttle, label);
                }
            }
            if control.sustained_alerts {
                let notices = engine.check(
                    snapshot.timestamp_ms,
                    &snapshot.core,
                    charge,
                    &control.alerts,
                    &control.ping_target,
                );
                for notice in notices {
                    push(HistoryEventKind::Alert, notice.body.clone());
                    let shown = app
                        .notification()
                        .builder()
                        .title(notice.title)
                        .body(notice.body)
                        .show();
                    if let Err(e) = shown {
                        crate::diagnostics::log_error(format_args!("notify: {e}"));
                    }
                }
            } else {
                engine.reset();
            }
            self.record_events(events, ts_ms, control.history_hours);
            self.history
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .record(
                    snapshot.timestamp_ms,
                    &snapshot.core,
                    charge.map(|c| c.0),
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
