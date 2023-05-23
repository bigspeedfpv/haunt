// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use color_eyre::eyre::Result;
use futures::lock::Mutex;
use std::sync::Arc;

mod api;

#[derive(Default)]
struct InnerState {
    http: reqwest::Client,

    match_id: Option<String>,

    lockfile_config: Option<api::lockfile::Config>,
    entitlements_config: Option<api::local::entitlements::Config>,
    session_config: Option<api::local::sessions::Config>,
}

pub struct HauntState(Arc<Mutex<InnerState>>);

fn main() -> Result<()> {
    color_eyre::install()?;

    env_logger::builder()
        .filter(Some("haunt"), log::LevelFilter::Debug)
        .init();

    let http = reqwest::Client::builder()
        .build()
        .expect("failed to build reqwest client");

    tauri::Builder::default()
        .manage(HauntState(Arc::new(Mutex::new(InnerState {
            http,
            ..Default::default()
        }))))
        .invoke_handler(tauri::generate_handler![api::check_user_data])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
