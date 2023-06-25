use crate::api;

#[derive(Debug, thiserror::Error)]
pub enum LoginFail {
    #[error("Unable to load lockfile config. Valorant probably isn't running.")]
    Lockfile,
    #[error("Unable to load entitlements config. Valorant probably isn't running.")]
    Entitlements,
    #[error("Unable to load session.")]
    Session,
}

impl serde::Serialize for LoginFail {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(match self {
            LoginFail::Lockfile => 0,
            LoginFail::Entitlements => 1,
            LoginFail::Session => 2,
        })
    }
}

// Returns a Result. Err signifies the stage login failed at.
#[tauri::command]
pub async fn login(state: tauri::State<'_, crate::HauntState>) -> Result<(), LoginFail> {
    log::info!("Loading lockfile...");

    let lockfile_config = api::lockfile::load_config();

    let lockfile_config = match lockfile_config {
        Some(config) => {
            log::info!("Lockfile config loaded successfully.");
            log::debug!("{:#?}", config);
            let mut lockfile_config = state.0.lockfile_config.lock().await;
            *lockfile_config = Some(config.clone());
            config
        }
        None => {
            log::error!("Unable to load lockfile config. Valorant probably isn't running.");
            return Err(LoginFail::Lockfile);
        }
    };

    log::info!("Loading entitlements...");

    let entitlements_config =
        api::local::entitlements::login(&lockfile_config, &state.0.offline_http).await;
    match entitlements_config {
        Ok(config) => {
            log::info!("Entitlements config loaded successfully.");
            log::debug!("{:#?}", config);
            let mut entitlements_config = state.0.entitlements_config.lock().await;
            *entitlements_config = Some(config.clone());
        }
        Err(e) => {
            log::error!("Unable to load entitlements config: {e}");
            return Err(LoginFail::Entitlements);
        }
    };

    log::info!("Loading session...");

    let session_config =
        api::local::sessions::load_config(&lockfile_config, &state.0.offline_http, &state.0.http)
            .await;
    let session_config = match session_config {
        Ok(config) => {
            log::info!("Session config loaded successfully.");
            log::debug!("{:#?}", config);
            let mut session_config = state.0.session_config.lock().await;
            *session_config = Some(config.clone());
            config
        }
        Err(why) => {
            log::error!("Unable to load session config: {why}");
            return Err(LoginFail::Session);
        }
    };

    log::info!("Getting player's username...");
    let presences =
        api::local::presence::get_presences(&lockfile_config, &state.0.offline_http).await;
    if let Ok(presences) = presences {
        let user = presences
            .iter()
            .find(|presence| presence.puuid == session_config.puuid)
            .unwrap();
        log::info!("Playing as {}", user.game_name);
    } else {
        return Err(LoginFail::Session);
    }

    Ok(())
}

#[derive(serde::Serialize)]
pub struct Match {}

#[tauri::command]
pub async fn load_match(state: tauri::State<'_, crate::HauntState>) -> Result<Match, ()> {
    let lockfile_config = state.0.lockfile_config.lock().await;
    // rust memory is wild bruh &*??????
    let lockfile_config = match &*lockfile_config {
        Some(config) => config,
        None => {
            log::error!("No lockfile config set. Was load_match called before login?");
            return Err(());
        }
    };

    let entitlements_config = state.0.entitlements_config.lock().await;
    let entitlements_config = match &*entitlements_config {
        Some(config) => config,
        None => {
            log::error!("No entitlements config set. Was load_match called before login?");
            return Err(());
        }
    };

    let session_config = state.0.session_config.lock().await;
    let session_config = match &*session_config {
        Some(config) => config,
        None => {
            log::error!("No session config set. Was load_match called before login?");
            return Err(());
        }
    };

    log::info!("Checking if player is in a match...");

    // we don't actually need the match id but it's a quick way to check ingame status
    let match_id =
        api::pvp::find_match_id(&entitlements_config, &state.0.http, &session_config).await;
    let match_id = match match_id {
        Some(id) => {
            log::info!("Player is in a match.");
            log::debug!("Match ID: {}", id);
            id
        }
        None => {
            log::info!("Player is not in a match.");
            return Err(());
        }
    };

    log::info!("Loading player presences...");

    let players =
        api::local::presence::get_presences(&lockfile_config, &state.0.offline_http).await;
    let players = match players {
        Ok(players) => players,
        Err(why) => {
            log::error!("Unable to load player presences: {why}");
            return Err(());
        }
    };

    let match_info = api::local::presence::get_match_info(&session_config.puuid, &players).await;
    log::debug!("Match info: {:#?}", match_info);

    log::info!("Loading player info.");
    api::player::debug_parties(players);

    let seasons = super::valapi::seasons::get_prev_3_seasons(&state).await;
    let seasons = match &seasons {
        // we want the last 3 seasons plus the current one
        Ok(seasons) => &seasons[..=3],
        Err(why) => {
            log::error!("Unable to load seasons: {:#?}", why);
            return Err(());
        }
    };

    log::debug!(
        "Past 3 seasons: {:#?}",
        seasons.iter().map(|s| &s.season_uuid).collect::<Vec<_>>()
    );

    let match_data = api::pvp::matchdata::get_match_info(
        &session_config,
        &entitlements_config,
        &match_id,
        &state.0.http,
    )
    .await;
    let match_data = match match_data {
        Ok(match_data) => match_data,
        Err(why) => {
            log::error!("Unable to load match players: {:#?}", why);
            return Err(());
        }
    };

    for mut player in match_data.players {
        log::info!("Filling history for player {}", player.puuid);
        let history = api::pvp::mmr::get_player_history(
            &player.puuid,
            &state.0.http,
            &entitlements_config,
            &session_config,
            seasons,
        )
        .await;
        let history = match history {
            Ok(history) => history,
            Err(why) => {
                log::warn!(
                    "Unable to load player history: {:#?}. Using empty Vec.",
                    why
                );
                Vec::new()
            }
        };
        player.competitive_history = history;
    }

    Ok(Match {})
}
