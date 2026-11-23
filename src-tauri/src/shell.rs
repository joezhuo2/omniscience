//! App-level state: settings, overlay visibility (user hide vs fullscreen hide), autostart and the
//! global hotkey. The tray, commands and the fullscreen watcher all go through here.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::menu::{CheckMenuItem, Menu, MenuItem, Submenu};
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
    pub menu: Menu<Wry>,
    pub system_updates: MenuItem<Wry>,
    pub profiles: Submenu<Wry>,
    pub snooze: MenuItem<Wry>,
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_default()
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
            let snoozed = s.alerts.snooze_until_ms > now_ms();
            let _ = t.snooze.set_text(if snoozed {
                "Resume alerts"
            } else {
                "Snooze alerts for 1 hour"
            });
            let shown = t.menu.get(t.system_updates.id()).is_some();
            if s.config_enabled && !shown {
                let _ = t.menu.insert(&t.system_updates, 8);
                let _ = t.menu.insert(&t.profiles, 8);
            } else if !s.config_enabled && shown {
                let _ = t.menu.remove(&t.system_updates);
                let _ = t.menu.remove(&t.profiles);
            }
            for item in t.profiles.items().unwrap_or_default() {
                let _ = t.profiles.remove(&item);
            }
            for p in &s.profiles {
                let id = format!("{}{}", crate::profiles::MENU_PREFIX, p.id);
                if let Ok(item) = CheckMenuItem::with_id(
                    t.profiles.app_handle(),
                    id,
                    &p.name,
                    true,
                    p.id == s.active_profile,
                    None::<&str>,
                ) {
                    let _ = t.profiles.append(&item);
                }
            }
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
                crate::diagnostics::log_error(format_args!("overlay visibility: {e}"));
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
        let mut new = new.sanitized();
        if new.integrations.api_enabled && new.integrations.api_token.is_empty() {
            new.integrations.api_token = settings::new_token();
        }
        let old = self.settings();

        if new.launch_at_startup != old.launch_at_startup {
            if new.launch_at_startup && tauri::is_dev() {
                return Err("launch at startup needs an installed build, not a dev build".into());
            }
            let autolaunch = app.autolaunch();
            let r = if new.launch_at_startup {
                autolaunch.enable()
            } else {
                autolaunch.disable()
            };
            r.map_err(|e| format!("launch at startup: {e}"))?;
        }
        crate::profiles::validate_hotkeys(&new.profiles, &new.hotkey)?;
        if new.hotkey != old.hotkey {
            register_hotkey(app, &new.hotkey)?;
        }
        if let Err(e) = crate::profiles::sync_hotkeys(app, &old.profiles, &new.profiles) {
            if new.hotkey != old.hotkey {
                let _ = app.global_shortcut().unregister(new.hotkey.as_str());
            }
            return Err(e);
        }
        if new.hotkey != old.hotkey {
            let _ = app.global_shortcut().unregister(old.hotkey.as_str());
        }
        app.state::<Arc<Telemetry>>().update(|c| new.apply(c));
        let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
        settings::save(&dir, &new)?;
        crate::config_gate::set(new.config_enabled);
        *self.settings.lock().unwrap_or_else(|e| e.into_inner()) = new.clone();
        crate::api_server::sync(app, &new.integrations);
        crate::mqtt::sync(&app.state::<Arc<Telemetry>>(), &new.integrations);
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
        if launch && !tauri::is_dev() {
            // Repoint the login entry at this exe in case another copy registered its own path.
            if let Err(e) = app.autolaunch().enable() {
                crate::diagnostics::log_error(format_args!("launch at startup: {e}"));
            }
        }
        let (hotkey, profiles) = {
            let mut s = self.settings.lock().unwrap_or_else(|e| e.into_inner());
            s.launch_at_startup = launch;
            (s.hotkey.clone(), s.profiles.clone())
        };
        if let Err(e) = register_hotkey(app, &hotkey) {
            crate::diagnostics::log_error(e);
        }
        if let Err(e) = crate::profiles::sync_hotkeys(app, &[], &profiles) {
            crate::diagnostics::log_error(e);
        }
        crate::profiles::spawn_watcher(app.clone());
        let integrations = self.settings().integrations;
        crate::api_server::sync(app, &integrations);
        crate::mqtt::sync(&app.state::<Arc<Telemetry>>(), &integrations);
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
