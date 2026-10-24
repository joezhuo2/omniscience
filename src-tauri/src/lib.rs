mod commands;
mod fullscreen;
mod hover;
mod platform;
mod positions;
mod settings;
mod shell;
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
        .invoke_handler(tauri::generate_handler![
            commands::set_expanded,
            commands::set_overlay_height,
            commands::kill_process_group,
            commands::get_settings,
            commands::set_settings,
            commands::reset_position
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
