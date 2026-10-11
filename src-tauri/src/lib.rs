mod platform;
mod tray;
mod window;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let main = app
                .get_webview_window("main")
                .expect("window \"main\" is declared in tauri.conf.json");
            platform::prepare_overlay(&main);
            window::place_top_center(&main)?;
            tray::init(app.handle())?;
            main.show()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Omniscience");
}
