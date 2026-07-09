#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    app::launch(tauri::generate_context!()).expect("error while running tauri application");
}
