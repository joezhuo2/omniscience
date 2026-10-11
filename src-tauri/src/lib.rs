mod commands;
mod platform;
mod telemetry;
mod tray;
mod window;

use tauri::Manager;

pub fn run() {
    let telemetry = telemetry::Telemetry::new(telemetry::Control::default());
    tauri::Builder::default()
        .manage(telemetry.clone())
        .invoke_handler(tauri::generate_handler![
            commands::set_expanded,
            commands::kill_process_group
        ])
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let main = app
                .get_webview_window("main")
                .expect("window \"main\" is declared in tauri.conf.json");
            platform::prepare_overlay(&main);
            window::place_top_center(&main)?;
            tray::init(app.handle())?;
            main.show()?;
            telemetry.start(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Omniscience");
}
