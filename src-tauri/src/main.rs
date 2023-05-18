// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use futures::lock::Mutex;
use std::sync::Arc;

mod api;

#[derive(Default)]
pub struct HauntState {
    lockfile_config: Arc<Mutex<Option<api::lockfile::Config>>>,
    session_config: Arc<Mutex<Option<api::local::sessions::Config>>>,
}

fn main() {
    env_logger::builder()
        .filter(Some("haunt"), log::LevelFilter::Info)
        .init();

    tauri::Builder::default()
        .manage(HauntState::default())
        .invoke_handler(tauri::generate_handler![api::check_user_data])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
