pub mod local;
pub mod lockfile;

#[tauri::command]
pub async fn check_user_data(state: tauri::State<'_, crate::HauntState>) -> Result<(), ()> {
    log::info!("Loading lockfile...");

    let lockfile_config = lockfile::load_config();

    if lockfile_config.is_none() {
        log::error!("Unable to load lockfile config. Valorant is probably not running.");
        return Err(());
    }

    let lockfile_config = lockfile_config.unwrap();
    log::info!("Lockfile config loaded: {:#?}", lockfile_config);

    log::info!("Loading session...");

    let session_config = local::sessions::load_config(lockfile_config).await;

    match session_config {
        Ok(config) => {
            log::info!("Session config loaded: {:#?}", config);
        }
        Err(why) => {
            log::error!("Unable to load session config: {why}");
        }
    }

    Ok(())
}
