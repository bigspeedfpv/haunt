use serde::Serialize;

use crate::api::{
    self,
    pvp::matchdata::{MatchData, Player},
    valapi::agents::Agent,
};

#[tauri::command]
pub async fn load_match(state: tauri::State<'_, crate::HauntState>) -> Result<ShortMatchData, ()> {
    let lockfile_config = state.0.lockfile_config.lock().await;
    // rust memory is wild bruh &*??????
    let Some(lockfile_config) = &*lockfile_config else {
        error!("No lockfile config set. Was load_match called before login?");
        return Err(());
    };

    let entitlements_config = state.0.entitlements_config.lock().await;
    let Some(entitlements_config) = &*entitlements_config else {
        error!("No entitlements config set. Was load_match called before login?");
        return Err(());
    };

    let session_config = state.0.session_config.lock().await;
    let Some(session_config) = &*session_config else {
        error!("No session config set. Was load_match called before login?");
        return Err(());
    };

    info!("Checking if player is in a match...");

    // we don't actually need the match id but it's a quick way to check ingame status
    let match_id =
        api::pvp::find_match_id(&entitlements_config, &state.0.http, &session_config).await;
    let match_id = match match_id {
        Some(id) => {
            info!("Player is in a match.");
            debug!("Match ID: {}", id);
            id
        }
        None => {
            info!("Player is not in a match.");
            return Err(());
        }
    };

    info!("Loading player presences...");

    let players =
        api::local::presence::get_presences(&lockfile_config, &state.0.offline_http).await;
    let players = match players {
        Ok(players) => players,
        Err(why) => {
            error!("Unable to load player presences: {why}");
            return Err(());
        }
    };

    let match_info = api::local::presence::get_match_info(&session_config.puuid, &players).await;
    debug!("Match info: {:#?}", match_info);

    info!("Loading player info.");
    api::player::debug_parties(players);

    let seasons = api::valapi::seasons::get_prev_3_seasons(&state).await;
    let seasons = match &seasons {
        // we want the last 3 seasons plus the current one
        Ok(seasons) => &seasons[..=3],
        Err(why) => {
            error!("Unable to load seasons: {:#?}", why);
            return Err(());
        }
    };

    debug!(
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
    let mut match_data = match match_data {
        Ok(match_data) => match_data,
        Err(why) => {
            error!("Unable to load match players: {:#?}", why);
            return Err(());
        }
    };

    let names = api::pvp::names::load_player_names(
        &match_data.players,
        &session_config,
        &entitlements_config,
        &state.0.http,
    )
    .await;
    let Ok(names) = names else {
        error!("Unable to load player names.");
        return Err(());
    };
    for name in names {
        let player = match_data
            .players
            .iter_mut()
            .find(|p| p.puuid == name.0)
            .unwrap(); // player exists because we got it from this Vec

        player.set_name(name.1);
    }

    debug!("Filling match history with acts: {:#?}", seasons);

    for player in &mut match_data.players {
        info!("Filling history for player {}", player.puuid);
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
                warn!(
                    "Unable to load player history: {:#?}. Using empty Vec.",
                    why
                );
                Vec::new()
            }
        };
        player.competitive_history = history;
    }

    // prefetched list of agents, mapped to uuid
    let agents = &state.0.agents;

    Ok(ShortMatchData::from_match_data(match_data, agents))
}

#[derive(Debug, Serialize)]
pub struct ShortMatchData {
    pub map: String,
    pub mode: String,
    pub players: Vec<ShortPlayer>,
}

#[derive(Debug, Serialize)]
pub struct ShortPlayer {
    pub name: String,
    pub team: String,
    pub character: Option<Agent>,
    pub title: String,
    #[serde(rename = "accountLevel")]
    pub account_level: Option<u32>,
    #[serde(rename = "rankHistory")]
    pub rank_history: Vec<u32>,
}

impl ShortPlayer {
    fn from_player(value: &Player, agents: &Vec<Agent>) -> Self {
        ShortPlayer {
            name: value.get_name(agents),
            team: value.team.to_string(),
            character: value.get_agent(agents),
            title: value.title.clone(),
            account_level: value.get_account_level(),
            rank_history: value
                .competitive_history
                .iter()
                .map(|a| a.competitive_tier)
                .collect(),
        }
    }
}

impl ShortMatchData {
    fn from_match_data(value: MatchData, agents: &Vec<Agent>) -> Self {
        ShortMatchData {
            map: value.map,
            mode: value.mode,
            players: value
                .players
                .iter()
                .map(|p| ShortPlayer::from_player(p, &agents))
                .collect(),
        }
    }
}
