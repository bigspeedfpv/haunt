use color_eyre::Result;

use crate::api::local::{entitlements, sessions};

use serde::{Deserialize, Serialize};

mod ingame;
mod pregame;

#[derive(Debug, Serialize)]
pub struct MatchData {
    pub map: String,
    pub mode: String,
    pub players: Vec<Player>,
}

#[derive(Debug, Serialize)]
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
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Team {
    Blue,
    Red,
    Other(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Character {
    None,
    Hovered(String),
    Locked(String),
}

impl Player {
    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn get_name(&self) -> String {
        match self.incognito {
            true => "Player",
            false => &self.name,
        }.into()
    }

    fn get_account_level(&self) -> u32{
        match self.hide_account_level {
            true => 0,
            false => self.account_level,
        }
    }

    fn get_agent(&self) -> Option<String> {
        match self.character {
            Character::None => None,
            Character::Hovered(ref agent) => Some(agent.clone()),
            Character::Locked(ref agent) => Some(agent.clone()),
        }
    }
}

pub async fn get_match_info(
    session: &sessions::Config,
    entitlements: &entitlements::Config,
    match_id: &str,
    http: &reqwest::Client,
) -> Result<MatchData> {
    // check ingame first
    let info = ingame::load_match_info(session, entitlements, match_id, http).await;
    debug!("Ingame endpoint returned: {:#?}", info);
    if info.is_ok() {
        return info;
    }

    // otherwise check pregame
    let info = pregame::load_match_info(session, entitlements, match_id, http).await;
    debug!("Pregame endpoint returned: {:#?}", info);
    info
}

fn get_name(
    puuid: &str,
) -> String {
    "hi".into()
}
