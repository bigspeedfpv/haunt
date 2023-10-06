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

#[derive(Debug, serde::Serialize)]
pub struct LoginInfo {
    username: String,
    tag: String,
    uuid: String,
    #[serde(rename = "accountLevel")]
    account_level: u32,
    rank: String,
}

// Returns a Result. Err signifies the stage login failed at.
#[tauri::command]
pub async fn login(state: tauri::State<'_, crate::HauntState>) -> Result<LoginInfo, LoginFail> {
    info!("Loading lockfile...");

    let lockfile_config = api::lockfile::load_config();

    let lockfile_config = match lockfile_config {
        Some(config) => {
            info!("Lockfile config loaded successfully.");
            debug!("{:#?}", config);
            let mut lockfile_config = state.0.lockfile_config.lock().await;
            *lockfile_config = Some(config.clone());
            config
        }
        None => {
            error!("Unable to load lockfile config. Valorant probably isn't running.");
            return Err(LoginFail::Lockfile);
        }
    };

    info!("Loading entitlements...");

    let entitlements_config =
        api::local::entitlements::login(&lockfile_config, &state.0.offline_http).await;
    match entitlements_config {
        Ok(config) => {
            info!("Entitlements config loaded successfully.");
            debug!("{:#?}", config);
            let mut entitlements_config = state.0.entitlements_config.lock().await;
            *entitlements_config = Some(config.clone());
        }
        Err(e) => {
            error!("Unable to load entitlements config: {e}");
            warn!("This is probably not an issue with Haunt! Valorant is probably not running.");
            return Err(LoginFail::Entitlements);
        }
    };

    info!("Loading session...");

    let session_config =
        api::local::sessions::load_config(&lockfile_config, &state.0.offline_http, &state.0.http)
            .await;
    let session_config = match session_config {
        Ok(config) => {
            info!("Session config loaded successfully.");
            debug!("{:#?}", config);
            let mut session_config = state.0.session_config.lock().await;
            *session_config = Some(config.clone());
            config
        }
        Err(why) => {
            error!("Unable to load session config: {why}");
            warn!("It looks like Valorant isn't logged in.");
            return Err(LoginFail::Session);
        }
    };

    info!("Getting player's username...");
    let presences =
        api::local::presence::get_presences(&lockfile_config, &state.0.offline_http).await;
    match presences {
        Ok(presences) => {
            let user = presences
                .iter()
                .find(|presence| presence.puuid == session_config.puuid);

            let Some(user) = user else {
                error!("Unable to find user in presences. User is probably not logged in.");
                return Err(LoginFail::Session);
            };

            info!("Playing as {}", user.game_name);
            Ok(LoginInfo {
                username: user.game_name.clone(),
                tag: user.game_tag.clone(),
                uuid: user.puuid.clone(),
                account_level: user.private.account_level,
                rank: user.private.competitive_tier.to_string(),
            })
        }
        Err(_) => Err(LoginFail::Session),
    }
}
