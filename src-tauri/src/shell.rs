//! App-level state: settings, overlay visibility (user hide vs fullscreen hide), autostart and the
//! global hotkey. The tray, commands and the fullscreen watcher all go through here.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::menu::CheckMenuItem;
use tauri::{AppHandle, Emitter, Manager, Wry};
use tauri_plugin_autostart::ManagerExt as _;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::settings::{self, Settings};
use crate::telemetry::Telemetry;
use crate::webview;

pub const SETTINGS_EVENT: &str = "omniscience://settings";
pub const OPEN_SETTINGS_EVENT: &str = "omniscience://open-settings";

/// Tray checkboxes that mirror settings.
pub struct TrayChecks {
    pub launch_at_startup: CheckMenuItem<Wry>,
    pub auto_check_updates: CheckMenuItem<Wry>,
    pub hide_over_fullscreen: CheckMenuItem<Wry>,
}

pub struct Shell {
    settings: Mutex<Settings>,
    user_hidden: AtomicBool,
    fullscreen_hidden: AtomicBool,
    tray: Mutex<Option<TrayChecks>>,
}

impl Shell {
    pub fn new(settings: Settings) -> Arc<Self> {
        Arc::new(Self {
            settings: Mutex::new(settings),
            user_hidden: AtomicBool::new(false),
            fullscreen_hidden: AtomicBool::new(false),
            tray: Mutex::new(None),
        })
    }

    pub fn settings(&self) -> Settings {
        self.settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn set_tray(&self, checks: TrayChecks) {
        *self.tray.lock().unwrap_or_else(|e| e.into_inner()) = Some(checks);
        self.sync_tray();
    }

    pub fn sync_tray(&self) {
        let s = self.settings();
        if let Some(t) = self.tray.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
            let _ = t.launch_at_startup.set_checked(s.launch_at_startup);
            let _ = t.auto_check_updates.set_checked(s.auto_check_updates);
            let _ = t.hide_over_fullscreen.set_checked(s.hide_over_fullscreen);
        }
    }

    /// Shows or hides the overlay from the user's two hide reasons, pausing telemetry and trimming
    /// WebView2 memory while hidden.
    fn apply_visibility(&self, app: &AppHandle) {
        let visible = !self.user_hidden.load(Ordering::SeqCst)
            && !self.fullscreen_hidden.load(Ordering::SeqCst);
        if let Some(window) = app.get_webview_window("main") {
            let result = if visible {
                window.show()
            } else {
                window.hide()
            };
            if let Err(e) = result {
                eprintln!("overlay visibility: {e}");
            }
            if !visible {
                webview::set_memory_low(&window, true);
            }
        }
        app.state::<Arc<Telemetry>>()
            .update(|c| c.visible = visible);
    }

    pub fn toggle(&self, app: &AppHandle) {
        self.user_hidden.fetch_xor(true, Ordering::SeqCst);
        self.apply_visibility(app);
    }

    pub fn show(&self, app: &AppHandle) {
        self.user_hidden.store(false, Ordering::SeqCst);
        self.apply_visibility(app);
    }

    fn set_fullscreen_hidden(&self, app: &AppHandle, hidden: bool) {
        if self.fullscreen_hidden.swap(hidden, Ordering::SeqCst) != hidden {
            self.apply_visibility(app);
        }
    }

    /// Validates, applies and saves new settings. Returns what was actually applied.
    pub fn update_settings(&self, app: &AppHandle, new: Settings) -> Result<Settings, String> {
        let new = new.sanitized();
        let old = self.settings();

        if new.launch_at_startup != old.launch_at_startup {
            let autolaunch = app.autolaunch();
            let r = if new.launch_at_startup {
                autolaunch.enable()
            } else {
                autolaunch.disable()
            };
            r.map_err(|e| format!("launch at startup: {e}"))?;
        }
        if new.hotkey != old.hotkey {
            register_hotkey(app, &new.hotkey)?;
            let _ = app.global_shortcut().unregister(old.hotkey.as_str());
        }
        app.state::<Arc<Telemetry>>().update(|c| new.apply(c));
        settings::save(app, &new)?;
        *self.settings.lock().unwrap_or_else(|e| e.into_inner()) = new.clone();
        self.sync_tray();
        if !new.hide_over_fullscreen {
            self.set_fullscreen_hidden(app, false);
        }
        let _ = app.emit(SETTINGS_EVENT, &new);
        Ok(new)
    }

    /// Applies loaded settings at startup (autostart state is read from the OS).
    pub fn init(self: &Arc<Self>, app: &AppHandle) {
        let launch = app.autolaunch().is_enabled().unwrap_or(false);
        let hotkey = {
            let mut s = self.settings.lock().unwrap_or_else(|e| e.into_inner());
            s.launch_at_startup = launch;
            s.hotkey.clone()
        };
        if let Err(e) = register_hotkey(app, &hotkey) {
            eprintln!("{e}");
        }
        self.spawn_fullscreen_watch(app.clone());
    }

    fn spawn_fullscreen_watch(self: &Arc<Self>, app: AppHandle) {
        let this = Arc::clone(self);
        std::thread::Builder::new()
            .name("fullscreen-watch".into())
            .spawn(move || {
                loop {
                    std::thread::sleep(Duration::from_secs(2));
                    if !this.settings().hide_over_fullscreen {
                        continue;
                    }
                    let Some(window) = app.get_webview_window("main") else {
                        continue;
                    };
                    #[cfg(windows)]
                    let over = window
                        .hwnd()
                        .map(|h| crate::fullscreen::fullscreen_app_over(h.0 as isize))
                        .unwrap_or(false);
                    #[cfg(not(windows))]
                    let over = {
                        let _ = &window;
                        crate::fullscreen::fullscreen_app_over(0)
                    };
                    this.set_fullscreen_hidden(&app, over);
                }
            })
            .expect("spawn fullscreen watcher");
    }
}

fn register_hotkey(app: &AppHandle, hotkey: &str) -> Result<(), String> {
    app.global_shortcut()
        .on_shortcut(hotkey, |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                app.state::<Arc<Shell>>().toggle(app);
            }
        })
        .map_err(|e| format!("hotkey {hotkey}: {e}"))
}
