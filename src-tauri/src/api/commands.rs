use crate::api;

#[derive(Debug, thiserror::Error)]
pub enum LoginFail {
    #[error("Unable to load lockfile config. Valorant probably isn't running.")]
    Lockfile,
    #[error("Unable to load session config. Valorant probably isn't running.")]
    Session,
    #[error("Player is not in a match.")]
    Match,
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
pub async fn login(state: tauri::State<'_, crate::HauntState>) -> Result<(), LoginFail> {
    log::info!("Loading lockfile...");

    let lockfile_config = api::lockfile::load_config();

    match &lockfile_config {
        Some(config) => {
            log::info!("Lockfile config loaded successfully.");
            log::debug!("{:#?}", config);
            let mut state_guard = state.0.lock().await;
            state_guard.lockfile_config = Some(config.clone());
        }
        None => {
            log::error!("Unable to load lockfile config. Valorant probably isn't running.");
            return Err(LoginFail::Lockfile);
        }
    }

    log::info!("Loading entitlements...");

    let entitlements_config = api::local::entitlements::login(&state).await;

    match &entitlements_config {
        Ok(config) => {
            log::info!("Entitlements config loaded successfully.");
            log::debug!("{:#?}", config);
            let mut state_guard = state.0.lock().await;
            state_guard.entitlements_config = Some(config.clone());
        }
        Err(e) => {
            log::error!("Unable to load entitlements config: {e}");
            return Err(LoginFail::Session);
        }
    }

    log::info!("Loading session...");

    let session_config = api::local::sessions::load_config(&state).await;

    let session = match &session_config {
        Ok(config) => {
            log::info!("Session config loaded successfully.");
            log::debug!("{:#?}", config);
            config.clone()
        }
        Err(why) => {
            log::error!("Unable to load session config: {why}");
            return Err(LoginFail::Session);
        }
    };

    log::info!("Checking if player is in a match...");

    let match_id = api::pvp::find_match_id(&state, session).await;

    let match_id = match &match_id {
        Some(id) => {
            log::info!("Player is in a match.");
            log::debug!("Match ID: {}", id);
            id.clone()
        }
        None => {
            log::info!("Player is not in a match.");
            return Err(LoginFail::Match);
        }
    };

    log::info!("Loading player presences...");

    let players = api::local::presence::get_match_players(&state).await;

    if players.is_err() {
        log::error!("Unable to load player presences.");
        return Err(LoginFail::Match);
    }

    let players = players.unwrap();

    let match_info =
        api::local::presence::get_match_info(session_config.unwrap().puuid, &players).await;
    log::debug!("Match info: {:#?}", match_info);

    log::info!("Loading player info.");
    api::player::debug_parties(players);

    let seasons = super::valapi::seasons::get_prev_3_seasons(&state).await;
    let seasons = match &seasons {
        Ok(seasons) => {
            &seasons[..3]
        }
        Err(why) => {
            log::error!("Unable to load seasons: {:#?}", why);
            return Err(LoginFail::Match);
        }
    };

    log::debug!("Past 3 seasons: {:#?}", seasons.iter().map(|s| &s.uuid).collect::<Vec<_>>());

    Ok(())
}

#[allow(unused)]
#[tauri::command]
async fn load_players(
    state: &tauri::State<'_, crate::HauntState>,
) -> Result<Vec<api::local::presence::Player>, ()> {
    Ok(vec![])
}
