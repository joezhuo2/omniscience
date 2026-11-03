#[cfg(windows)]
mod asus;
mod battery_saver;
mod charge_limit;
mod commands;
mod config_gate;
mod cpu_boost;
mod diagnostics;
mod display;
mod driver_updates;
mod fullscreen;
mod gpu_mode;
mod hover;
mod platform;
mod positions;
mod power_limits;
mod power_mode;
mod settings;
mod shell;
mod system_command;
mod telemetry;
mod tray;
mod webview;
mod window;

use std::sync::Arc;

use tauri::Manager;

pub fn run() {
    let telemetry = telemetry::Telemetry::new(telemetry::Control::default());
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // A second launch just reveals the running overlay.
            app.state::<Arc<shell::Shell>>().show(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostarted"]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(telemetry.clone())
        .manage(Arc::new(driver_updates::UpdateChecker::default()))
        .invoke_handler(tauri::generate_handler![
            commands::set_expanded,
            commands::set_overlay_height,
            commands::kill_process_group,
            commands::get_settings,
            commands::set_settings,
            commands::reset_position,
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
            battery_saver::get_battery_saver,
            battery_saver::set_battery_saver_threshold,
            battery_saver::set_battery_saver,
            display::get_display_mode,
            display::set_display_refresh,
            display::set_display_resolution,
            charge_limit::get_charge_limit,
            charge_limit::set_charge_limit
        ])
        .on_window_event(|window, event| {
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

            let settings = settings::load(app.handle());
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
            telemetry.start(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Omniscience");
}
