#![allow(unused)]
use std::{io::Write, path::PathBuf};

use color_eyre::eyre::Result;
use tauri::Manager;

pub mod seasons;
mod urls;

#[tauri::command]
pub async fn update_files(app_handle: tauri::AppHandle) -> Result<(), ()> {
    let data_dir = app_handle
        .path()
        .app_local_data_dir()
        .unwrap()
        .join("ValAPI");

    Ok(())
}

async fn update_version(data_dir: PathBuf) {
    let url = format!("{}{}", urls::BASE_URL, urls::VERSION.url);

    let version = reqwest::get(url).await;

    match version {
        Ok(version) => {
            let version = version.text().await.unwrap();
            let version_file = data_dir.join(urls::VERSION.suffix);
            let mut file = std::fs::File::create(version_file).unwrap();
            file.write_all(version.as_bytes()).unwrap();
        }
        Err(why) => {
            warn!("Unable to update version: {why}");
        }
    }
}

async fn update_maps(data_dir: PathBuf) {
    let url = format!("{}{}", urls::BASE_URL, urls::MAPS.url);

    let maps = reqwest::get(url).await;

    match maps {
        Ok(maps) => {
            let maps = maps.text().await.unwrap();
            let maps_file = data_dir.join(urls::MAPS.suffix);
            let mut file = std::fs::File::create(maps_file).unwrap();
            file.write_all(maps.as_bytes()).unwrap();
        }
        Err(why) => {
            warn!("Unable to update maps: {why}");
        }
    }
}
