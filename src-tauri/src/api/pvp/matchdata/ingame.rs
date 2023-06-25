use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::api::local::{entitlements, sessions};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct MatchInfo {
    #[serde(rename = "MapID")]
    map_id: String,
    #[serde(rename = "ModeID")]
    mode_id: String,
    players: Vec<MatchPlayer>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct MatchPlayer {
    #[serde(rename = "Subject")]
    puuid: String,
    #[serde(rename = "TeamID")]
    team: super::Team,
    #[serde(rename = "CharacterID")]
    character_id: String,
    player_identity: PlayerIdentity,
    is_coach: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlayerIdentity {
    #[serde(rename = "PlayerCardID")]
    card_id: String,
    #[serde(rename = "PlayerTitleID")]
    title_id: String,
    account_level: u32,
    #[serde(rename = "PreferredLevelBorderID")]
    border_id: String,
    incognito: bool,
    hide_account_level: bool,
}

impl Into<super::MatchData> for MatchInfo {
    fn into(self) -> super::MatchData {
        // we don't care about coach ranks lol
        let players = self
            .players
            .into_iter()
            .filter(|p| !p.is_coach)
            .map(|p| p.into())
            .collect();

        super::MatchData {
            map: self.map_id,
            mode: self.mode_id,
            players,
        }
    }
}

// holy (?????)
impl Into<super::Player> for MatchPlayer {
    fn into(self) -> super::Player {
        super::Player {
            puuid: self.puuid,
            name: "".to_string(),
            team: self.team,
            character: super::Character::Locked(self.character_id),
            card: self.player_identity.card_id,
            title: self.player_identity.title_id,
            account_level: self.player_identity.account_level,
            border: self.player_identity.border_id,
            incognito: self.player_identity.incognito,
            hide_account_level: self.player_identity.hide_account_level,
            competitive_history: Vec::new(),
        }
    }
}

pub async fn load_match_info(
    session: &sessions::Config,
    entitlements: &entitlements::Config,
    match_id: &str,
    http: &reqwest::Client,
) -> Result<super::MatchData> {
    // first check ingame
    let endpoint = format!(
        "https://glz-{}-1.{}.a.pvp.net/core-game/v1/matches/{match_id}",
        session.region.to_string(),
        session.shard.to_string()
    );

    let info = http
        .get(endpoint)
        .bearer_auth(&entitlements.token)
        .header("X-Riot-Entitlements-JWT", &entitlements.jwt)
        .send()
        .await?
        .json::<MatchInfo>()
        .await?;

    Ok(info.into())
}
