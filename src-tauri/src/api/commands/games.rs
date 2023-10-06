use serde::Serialize;

use crate::api::{
    self,
    local::{entitlements, sessions, presence},
    lockfile,
    pvp::matchdata::{MatchData, Player},
    valapi::{agents::Agent, seasons::CompetitiveTier},
};

async fn load_configs(
    state: &tauri::State<'_, crate::HauntState>,
) -> Result<(lockfile::Config, entitlements::Config, sessions::Config), bool> {
    let lockfile_config = state.0.lockfile_config.lock().await;
    // rust memory is wild bruh &*??????
    let Some(lockfile_config) = &*lockfile_config else {
        error!("No lockfile config set. Was load_match called before login?");
        return Err(false);
    };

    let entitlements_config = state.0.entitlements_config.lock().await;
    let Some(entitlements_config) = &*entitlements_config else {
        error!("No entitlements config set. Was load_match called before login?");
        return Err(false);
    };

    let session_config = state.0.session_config.lock().await;
    let Some(session_config) = &*session_config else {
        error!("No session config set. Was load_match called before login?");
        return Err(false);
    };

    Ok((
        lockfile_config.clone(),
        entitlements_config.clone(),
        session_config.clone(),
    ))
}

async fn refresh_login(
    state: &tauri::State<'_, crate::HauntState>,
    lockfile_config: &lockfile::Config,
    entitlements_config: &entitlements::Config,
    session_config: &sessions::Config,
) -> Result<(String, Vec<presence::Player>), bool> {
    info!("Ensuring correct user still logged in...");
    let players =
        api::local::presence::get_presences(&lockfile_config, &state.0.offline_http).await;

    let Ok(players) = players else {
        error!("Unable to load player presences.");
        return Err(false);
    };

    let user = players
        .iter()
        .find(|presence| presence.puuid == session_config.puuid);

    if user.is_none() {
        error!(
            "User is not logged in. They probably switched accounts since login was last called."
        );
        return Err(false);
    }

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
            return Err(true);
        }
    };

    Ok((match_id, players))
}

#[tauri::command]
pub async fn load_match(
    state: tauri::State<'_, crate::HauntState>,
) -> Result<ShortMatchData, bool> {
    let (lockfile_config, entitlements_config, session_config) = load_configs(&state).await?;

    let (match_id, players) = refresh_login(
        &state,
        &lockfile_config,
        &entitlements_config,
        &session_config,
    )
    .await?;

    let match_info = api::local::presence::get_match_info(&session_config.puuid, &players).await;
    debug!("Match info: {:#?}", match_info);

    info!("Loading player info.");
    api::player::debug_parties(&players);

    let seasons = api::valapi::seasons::get_prev_3_seasons(&state).await;
    let seasons = match &seasons {
        // we want the last 3 seasons plus the current one
        Ok(seasons) => &seasons[..=3],
        Err(why) => {
            error!("Unable to load seasons: {:#?}", why);
            return Err(true);
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
        &players,
        &state.0.http,
    )
    .await;
    let mut match_data = match match_data {
        Ok(match_data) => match_data,
        Err(why) => {
            error!("Unable to load match players: {:#?}", why);
            return Err(true);
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
        return Err(true);
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
    let tiers = &state.0.competitive_tiers;

    let short_match = ShortMatchData::from_match_data(match_data, agents, tiers);

    let mut match_cache = state.0.match_cache.lock().await;
    *match_cache = Some(short_match.clone());

    Ok(short_match)
}

#[tauri::command]
pub async fn quick_update_match(
    state: tauri::State<'_, crate::HauntState>,
) -> Result<ShortMatchData, bool> {
    let (lockfile_config, entitlements_config, session_config) = load_configs(&state).await?;

    let (match_id, players) = refresh_login(
        &state,
        &lockfile_config,
        &entitlements_config,
        &session_config,
    ).await?;

    let match_cache = &state.0.match_cache;
    let mut match_cache = match_cache.lock().await;

    // if we don't have a match cache, or it's from a stale match, throw the user back to pregame 
    // &mut *match_cache is actually just fucked up beyond human comprehension
    let match_cache = match &mut *match_cache {
        Some(match_cache) => match_cache,
        None => {
            info!("Invalid match cache. Returning to pregame.");
            return Err(true);
        }
    };

    let match_info = api::local::presence::get_match_info(&session_config.puuid, &players).await;
    debug!("Match info: {:#?}", match_info);

    let match_data = api::pvp::matchdata::get_match_info(
        &session_config,
        &entitlements_config,
        &match_id,
        &players,
        &state.0.http,
    )
    .await;
    let match_data = match match_data {
        Ok(match_data) => match_data,
        Err(why) => {
            error!("Unable to load match players: {:#?}", why);
            return Err(true);
        }
    };
    
    // player names won't change here. we're just refetching agent status, that's it

    // prefetched list of agents, mapped to uuid
    let agents = &state.0.agents;

    match_cache.update_with_match_data(match_data, agents);

    Ok(match_cache.clone())
}

#[derive(Clone, Debug, Serialize)]
pub struct ShortMatchData {
    pub ingame: bool,
    pub map: String,
    pub mode: String,
    pub players: Vec<ShortPlayer>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ShortPlayer {
    pub uuid: String,
    pub name: String,
    pub team: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character: Option<Agent>,
    pub title: String,
    #[serde(rename = "accountLevel", skip_serializing_if = "Option::is_none")]
    pub account_level: Option<u32>,
    #[serde(rename = "rankHistory")]
    pub rank_history: Vec<CompetitiveTier>,
    #[serde(rename = "partyId")]
    pub party_id: String,
}

impl ShortPlayer {
    fn from_player(value: &Player, agents: &Vec<Agent>, tiers: &Vec<CompetitiveTier>) -> Self {
        println!("{:#?}", value);
        ShortPlayer {
            uuid: value.puuid.clone(),
            name: value.get_name(agents),
            team: value.team.to_string(),
            character: value.get_agent(agents),
            title: value.title.clone(),
            account_level: value.get_account_level(),
            rank_history: value
                .competitive_history
                .iter()
                .map(|a| CompetitiveTier::from_act_tier(tiers, &a.episode_id, a.competitive_tier))
                .collect(),
            party_id: value.party_id.clone(),
        }
    }
}

impl ShortMatchData {
    fn from_match_data(
        value: MatchData,
        agents: &Vec<Agent>,
        tiers: &Vec<CompetitiveTier>,
    ) -> Self {
        ShortMatchData {
            ingame: value.ingame,
            map: value.map,
            mode: value.mode,
            players: value
                .players
                .iter()
                .map(|p| ShortPlayer::from_player(p, agents, tiers))
                .collect(),
        }
    }

    fn update_with_match_data(
        &mut self,
        value: MatchData,
        agents: &Vec<Agent>,
    ) {
        self.ingame = value.ingame;
        
        for player in &mut self.players {
            let updated_player = value.players.iter().find(|p| p.puuid == player.uuid);
            let Some(updated_player) = updated_player else {
                continue;
            };

            player.character = updated_player.get_agent(agents);
            player.party_id = updated_player.party_id.clone();
        }
    }
}
