// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate tracing;

use color_eyre::eyre::Result;
use futures::lock::Mutex;
use std::sync::Arc;
use tauri::Manager;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use window_vibrancy::{apply_acrylic, apply_mica};

mod api;
use api::commands::{self, ShortMatchData};

#[derive(Debug, Default)]
struct InnerState {
    http: reqwest::Client,
    offline_http: reqwest::Client, // used for local cnx with tls disabled

    agents: Vec<api::valapi::agents::Agent>,
    competitive_tiers: Vec<api::valapi::seasons::CompetitiveTier>,

    lockfile_config: Mutex<Option<api::lockfile::Config>>,
    entitlements_config: Mutex<Option<api::local::entitlements::Config>>,
    session_config: Mutex<Option<api::local::sessions::Config>>,

    match_cache: Mutex<Option<ShortMatchData>>,
}

// so we don't have to manually wrap each field in an Arc<T>
#[derive(Debug)]
pub struct HauntState(Arc<InnerState>);

fn main() -> Result<()> {
    // it do not matter if its not there gg
    _ = dotenvy::dotenv();

    color_eyre::install()?;

    tracing_subscriber::registry()
        .with(fmt::layer().pretty())
        .with(EnvFilter::from_default_env())
        .init();

    let http = reqwest::Client::builder()
        .build()
        .expect("failed to build reqwest client");

    let offline_http = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("failed to build reqwest client");

    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_window("main").unwrap();

            match apply_mica(&window, None) {
                Ok(_) => info!("Mica effect successfully applied"),
                Err(why) => {
                    warn!("Failed to apply Mica, falling back to Acrylic: {why}");
                    apply_acrylic(&window, None).expect("Failed to apply Mica and Acrylic.");
                }
            }

            window.minimize().unwrap();
            window.unminimize().unwrap();
            window.maximize().unwrap();
            window.unmaximize().unwrap();

            Ok(())
        })
        .manage(HauntState(Arc::new(InnerState {
            http,
            offline_http,

            agents: api::valapi::agents::load_agent_map(),
            competitive_tiers: api::valapi::seasons::load_competitive_tiers(),

            ..Default::default()
        })))
        .invoke_handler(tauri::generate_handler![
            commands::login,
            commands::load_match,
            commands::quick_update_match,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
