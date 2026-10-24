//! Telemetry engine: a single background thread samples sensors on a fixed interval and emits one
//! [`types::Snapshot`] per tick as the `telemetry://tick` event.

#[cfg(any(windows, test))]
mod parse;
mod peak;
mod ping;
pub mod processes;
mod sampler;
pub mod types;
#[cfg(windows)]
mod windows;

use std::panic::AssertUnwindSafe;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use tauri::{AppHandle, Emitter};

use types::Thresholds;

pub const TICK_EVENT: &str = "telemetry://tick";

/// What the collector should do; changed from commands, the tray and the window.
#[derive(Debug, Clone, PartialEq)]
pub struct Control {
    pub expanded: bool,
    pub visible: bool,
    pub interval: Duration,
    pub thresholds: Thresholds,
    /// Host to ping while expanded; empty for none.
    pub ping_target: Arc<str>,
}

impl Default for Control {
    fn default() -> Self {
        Self {
            expanded: false,
            visible: true,
            interval: Duration::from_secs(1),
            thresholds: Thresholds::default(),
            ping_target: "".into(),
        }
    }
}

pub struct Telemetry {
    control: Mutex<Control>,
    wake: Condvar,
}

impl Telemetry {
    pub fn new(control: Control) -> Arc<Self> {
        Arc::new(Self {
            control: Mutex::new(control),
            wake: Condvar::new(),
        })
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
                    eprintln!("telemetry collector panicked; restarting");
                    std::thread::sleep(Duration::from_secs(1));
                }
            })
            .expect("spawn telemetry thread");
    }

    fn run(&self, app: &AppHandle) {
        let mut sampler = sampler::Sampler::new();
        loop {
            let control = {
                let mut c = self.control.lock().unwrap_or_else(|e| e.into_inner());
                while !c.visible {
                    c = self.wake.wait(c).unwrap_or_else(|e| e.into_inner());
                }
                c.clone()
            };
            sampler.set_ping_target(control.ping_target.clone());
            let snapshot = sampler.sample(control.thresholds, control.expanded);
            if let Err(e) = app.emit(TICK_EVENT, &snapshot) {
                eprintln!("emit telemetry: {e}");
            }
            let c = self.control.lock().unwrap_or_else(|e| e.into_inner());
            // Woken early by any control change (e.g. expand), otherwise once per interval.
            let _ = self
                .wake
                .wait_timeout(c, control.interval)
                .unwrap_or_else(|e| e.into_inner());
        }
    }
}
