// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(error) = opendesk_lib::run() {
        eprintln!("error while running tauri application: {error}");
        std::process::exit(1);
    }
}
