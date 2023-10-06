use std::fmt::Display;

use color_eyre::Result;

use crate::api::{
    local::{entitlements, sessions},
    valapi::agents::Agent,
};

use serde::Deserialize;

mod ingame;
mod pregame;

#[derive(Debug)]
pub struct MatchData {
    pub ingame: bool,
    pub map: String,
    pub mode: String,
    pub players: Vec<Player>,
}

#[derive(Debug)]
pub struct Player {
    pub puuid: String,
    name: String,
    pub team: Team,
    character: Character,
    pub card: String,
    pub title: String,
    account_level: u32,
    pub border: String,
    incognito: bool,
    hide_account_level: bool,
    pub competitive_history: super::mmr::History,
    pub party_id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub enum Team {
    Blue,
    Red,
    Other(String),
}

impl Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Team::Blue => "blue",
            Team::Red => "red",
            Team::Other(_) => "unknown",
        };

        write!(f, "{}", s)
    }
}

#[derive(Debug, Deserialize)]
pub enum Character {
    None,
    Hovered(String),
    Locked(String),
}

impl Player {
    /// Used to fill names from the Valorant Name API.
    ///
    /// * `name` - The name to fill.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Returns the player's name, or an anonymized version if incognito.
    pub fn get_name(&self, agents: &Vec<Agent>) -> String {
        match self.incognito {
            true => self.get_incognito_name(agents),
            false => self.name.clone(),
        }
        .into()
    }

    fn get_incognito_name(&self, agents: &Vec<Agent>) -> String {
        match &self.character {
            Character::None => "Player".to_string(),
            Character::Hovered(ref c) | Character::Locked(ref c) => {
                match agent_from_uuid(agents, c) {
                    Some(agent) => agent.display_name,
                    None => "Player".to_string(),
                }
            }
        }
    }

    /// Returns the player's account level, or None if hidden.
    pub fn get_account_level(&self) -> Option<u32> {
        match self.hide_account_level {
            true => None,
            false => Some(self.account_level),
        }
    }

    /// Returns the player's agent, or None if not selected yet or agent unknown.
    ///
    /// * `agents` - The list of agents fetched from ValAPI.
    pub fn get_agent(&self, agents: &Vec<Agent>) -> Option<Agent> {
        trace!("Getting agent for {:?}", self.character);
        match self.character {
            Character::None => None,
            Character::Hovered(ref agent) | Character::Locked(ref agent) => {
                agent_from_uuid(agents, agent)
            }
        }
    }
}

pub async fn get_match_info(
    session: &sessions::Config,
    entitlements: &entitlements::Config,
    match_id: &str,
    presences: &Vec<crate::api::local::presence::Player>,
    http: &reqwest::Client,
) -> Result<MatchData> {
    // check ingame first
    let mut info = ingame::load_match_info(session, entitlements, match_id, http).await;
    debug!("Ingame endpoint returned: {:#?}", info);

    if info.is_err() {
        // otherwise check pregame
        info = pregame::load_match_info(session, entitlements, match_id, http).await;
        debug!("Pregame endpoint returned: {:#?}", info);
    }

    let Ok(mut info) = info else {
        return info; // bail if no match data
    };

    // map player party ids
    for player in &mut info.players {
        let presence = presences.iter().find(|p| p.puuid == player.puuid);
        let Some(presence) = presence else {
            warn!("Player {} not found in presences", player.puuid);
            continue;
        };

        player.party_id = presence.private.party_id.clone();
    }

    Ok(info)
}

fn agent_from_uuid(agents_map: &Vec<Agent>, uuid: &str) -> Option<Agent> {
    trace!(uuid = uuid, agents = ?agents_map, "Finding agent");
    agents_map.into_iter().find(|a| a.uuid == uuid).cloned()
}
