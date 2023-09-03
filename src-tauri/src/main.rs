// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate tracing;

use color_eyre::eyre::Result;
use futures::lock::Mutex;
use std::sync::Arc;

mod api;
use api::commands;

#[derive(Default)]
struct InnerState {
    http: reqwest::Client,
    offline_http: reqwest::Client, // used for local cnx with tls disabled

    lockfile_config: Mutex<Option<api::lockfile::Config>>,
    entitlements_config: Mutex<Option<api::local::entitlements::Config>>,
    session_config: Mutex<Option<api::local::sessions::Config>>,
}

// so we don't have to manually wrap each field in an Arc<T>
pub struct HauntState(Arc<InnerState>);

fn main() -> Result<()> {
    color_eyre::install()?;

    let format = tracing_subscriber::fmt::format()
        .pretty();
    tracing_subscriber::fmt().event_format(format).init();

    let http = reqwest::Client::builder()
        .build()
        .expect("failed to build reqwest client");

    let offline_http = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("failed to build reqwest client");

    tauri::Builder::default()
        .manage(HauntState(Arc::new(InnerState {
            http,
            offline_http,
            ..Default::default()
        })))
        .invoke_handler(tauri::generate_handler![
            commands::login,
            commands::load_match
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
