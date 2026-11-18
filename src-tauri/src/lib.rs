#[cfg(windows)]
#[cfg_attr(not(feature = "standalone"), allow(dead_code))]
mod asus;
#[cfg(feature = "standalone")]
mod battery_saver;
#[cfg(feature = "standalone")]
mod charge_limit;
mod commands;
#[cfg(feature = "standalone")]
mod config_gate;
#[cfg(feature = "standalone")]
mod cpu_boost;
mod diagnostics;
#[cfg(feature = "standalone")]
mod display;
#[cfg(feature = "standalone")]
mod display_toggles;
#[cfg(feature = "standalone")]
mod driver_updates;
pub mod embed;
#[cfg(feature = "standalone")]
mod fan_mode;
#[cfg(feature = "standalone")]
mod fullscreen;
#[cfg(feature = "standalone")]
mod gpu_mode;
#[cfg(feature = "standalone")]
mod hover;
#[cfg(feature = "standalone")]
mod platform;
#[cfg(feature = "standalone")]
mod positions;
#[cfg(feature = "standalone")]
mod power_limits;
#[cfg(feature = "standalone")]
mod power_mode;
#[cfg(feature = "standalone")]
mod profiles;
mod settings;
#[cfg(feature = "standalone")]
mod shell;
#[cfg(feature = "standalone")]
mod startup_apps;
#[cfg(feature = "standalone")]
mod system_command;
mod telemetry;
#[cfg(feature = "standalone")]
mod tray;
#[cfg(feature = "standalone")]
mod webview;
#[cfg(feature = "standalone")]
mod window;

#[cfg(feature = "standalone")]
use std::sync::Arc;

#[cfg(feature = "standalone")]
use tauri::Manager;

#[cfg(feature = "standalone")]
const AUTOSTART_ARG: &str = "--autostarted";

#[cfg(feature = "standalone")]
pub fn run() {
    // A stale login entry can point at a dev build, which has no bundled UI and opens a console.
    if tauri::is_dev() && std::env::args().any(|a| a == AUTOSTART_ARG) {
        return;
    }
    let telemetry = telemetry::Telemetry::new(telemetry::Control::default());
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // A second launch just reveals the running overlay.
            app.state::<Arc<shell::Shell>>().show(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![AUTOSTART_ARG]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(telemetry.clone())
        .manage(Arc::new(driver_updates::UpdateChecker::default()))
        .invoke_handler(tauri::generate_handler![
            commands::set_expanded,
            commands::set_overlay_height,
            commands::kill_process_group,
            commands::process_details,
            commands::open_process_location,
            commands::set_process_priority,
            commands::set_process_efficiency,
            commands::suspend_processes,
            commands::resume_processes,
            commands::kill_process_tree,
            commands::set_process_sampling,
            commands::open_widget,
            commands::set_widget_active,
            commands::get_settings,
            commands::set_settings,
            commands::reset_position,
            commands::get_telemetry_history,
            commands::get_history_events,
            commands::export_history,
            diagnostics::get_diagnostic_report,
            driver_updates::scan_driver_updates,
            driver_updates::get_system_update_status,
            driver_updates::open_update_download,
            gpu_mode::get_gpu_mode,
            gpu_mode::set_gpu_mode,
            power_limits::get_power_limits,
            power_limits::set_power_limits,
            power_limits::reset_power_limits,
            cpu_boost::get_cpu_boost,
            cpu_boost::set_cpu_boost,
            power_mode::get_power_mode,
            power_mode::set_power_mode,
            profiles::apply_profile,
            battery_saver::get_battery_saver,
            battery_saver::set_battery_saver_threshold,
            battery_saver::set_battery_saver,
            display::get_display_mode,
            display::set_display_refresh,
            display::set_display_resolution,
            charge_limit::get_charge_limit,
            charge_limit::set_charge_limit,
            fan_mode::get_fan_mode,
            fan_mode::set_fan_mode,
            display_toggles::get_display_toggles,
            display_toggles::set_hdr,
            display_toggles::set_night_light,
            display_toggles::set_brightness,
            startup_apps::get_startup_apps,
            startup_apps::set_startup_app_enabled
        ])
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::Destroyed) && window.label() == "widget" {
                let telemetry = window.app_handle().state::<Arc<telemetry::Telemetry>>();
                telemetry.set_process_viewer("widget", false);
                telemetry.update(|c| c.widget = false);
            }
            if let (tauri::WindowEvent::Moved(pos), Some(main)) =
                (event, window.app_handle().get_webview_window("main"))
                && window.label() == "main"
            {
                window
                    .app_handle()
                    .state::<Arc<positions::Positions>>()
                    .record(&main, *pos);
            }
        })
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let settings = app
                .path()
                .app_config_dir()
                .map(|dir| settings::load(&dir))
                .unwrap_or_default();
            config_gate::set(settings.config_enabled);
            app.manage(gpu_mode::GpuConfig::start(app.handle())?);
            app.manage(power_limits::PowerLimits::start(app.handle())?);
            app.manage(power_mode::PowerConfig::start(app.handle())?);
            app.manage(display::DisplayConfig::start(app.handle())?);
            app.manage(charge_limit::ChargeLimit::start(app.handle())?);
            telemetry.update(|c| settings.apply(c));
            let shell = shell::Shell::new(settings);
            app.manage(shell.clone());
            shell.init(app.handle());

            let main = app
                .get_webview_window("main")
                .expect("window \"main\" is declared in tauri.conf.json");
            platform::prepare_overlay(&main);
            let positions = Arc::new(positions::Positions::load(app.handle()));
            app.manage(positions.clone());
            window::place_top_center(&main)?;
            if let Some(saved) = positions.restore(&main) {
                main.set_position(saved)?;
            }
            positions.set_tracking(true);
            webview::set_memory_low(&main, true);
            main.set_ignore_cursor_events(true)?;
            hover::spawn(app.handle().clone());
            tray::init(app.handle())?;
            main.show()?;
            telemetry.open_history(app.path().app_data_dir().ok().as_deref());
            telemetry.start(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Omniscience");
}
