pub mod local;
pub mod lockfile;

#[derive(Debug, thiserror::Error)]
pub enum LoginFail {
    Lockfile,
    Session,
    Match,
}

impl std::fmt::Display for LoginFail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoginFail::Lockfile => write!(f, "Lockfile"),
            LoginFail::Session => write!(f, "Session"),
            LoginFail::Match => write!(f, "Match"),
        }
    }
}

impl serde::Serialize for LoginFail {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(match self {
            LoginFail::Lockfile => 0,
            LoginFail::Session => 1,
            LoginFail::Match => 2,
        })
    }
}

// Returns a Result. Err signifies the stage login failed at.
#[tauri::command]
pub async fn check_user_data(state: tauri::State<'_, crate::HauntState>) -> Result<(), LoginFail> {
    log::info!("Loading lockfile...");

    let lockfile_config = lockfile::load_config();

    match &lockfile_config {
        Some(config) => {
            log::info!("Lockfile config loaded: {:#?}", config);
            let mut lockfile_guard = state.lockfile_config.lock().await;
            *lockfile_guard = Some(config.clone());
        }
        None => {
            log::error!("Unable to load lockfile config. Valorant probably isn't running.");
            return Err(LoginFail::Lockfile);
        }
    }

    log::info!("Loading session...");

    let session_config = local::sessions::load_config(lockfile_config.unwrap()).await;

    match &session_config {
        Ok(config) => {
            log::info!("Session config loaded: {:#?}", config);
            let mut session_guard = state.session_config.lock().await;
            *session_guard = Some(config.clone());
        }
        Err(why) => {
            log::error!("Unable to load session config: {why}");
            return Err(LoginFail::Session);
        }
    }

    return Ok(());
}
