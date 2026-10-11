mod window;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let main = app
                .get_webview_window("main")
                .expect("window \"main\" is declared in tauri.conf.json");
            window::place_top_center(&main)?;
            main.show()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Omniscience");
}
