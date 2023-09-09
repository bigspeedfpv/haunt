use color_eyre::{Result, eyre::bail};
use serde::{Deserialize, Serialize};

use crate::api::local::{entitlements, sessions};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MatchInfo {
    #[serde(rename = "MapID")]
    map_id: String,
    mode: String,
    teams: Vec<MatchTeam>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MatchTeam {
    #[serde(rename = "TeamID")]
    team_id: super::Team,
    players: Vec<MatchPlayer>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MatchPlayer {
    #[serde(rename = "Subject")]
    puuid: String,
    character_selection_state: CharacterSelectionState,
    #[serde(rename = "CharacterID")]
    character_id: String,
    competitive_tier: u32,
    player_identity: PlayerIdentity,
    is_captain: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
enum CharacterSelectionState {
    Selected,
    Locked,
    #[serde(other)]
    None,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
        // SWEET MOTHER OF GOD
        let players = self
            .teams
            .iter()
            .map(|team| {
                let team = team.clone();
                team.players
                    .into_iter()
                    .filter(|p| !p.is_captain)
                    .map(move |p| super::Player {
                        puuid: p.puuid,
                        name: "".to_string(),
                        team: team.team_id.clone(),
                        character: get_character(p.character_selection_state, p.character_id),
                        card: p.player_identity.card_id,
                        title: p.player_identity.title_id,
                        account_level: p.player_identity.account_level,
                        border: p.player_identity.border_id,
                        incognito: p.player_identity.incognito,
                        hide_account_level: p.player_identity.hide_account_level,
                        competitive_history: Vec::new(),
                    })
            })
            .flatten()
            .collect();

        super::MatchData {
            ingame: false,
            map: self.map_id,
            mode: self.mode,
            players,
        }
    }
}

fn get_character(state: CharacterSelectionState, character: String) -> super::Character {
    match state {
        CharacterSelectionState::Locked => super::Character::Locked(character),
        CharacterSelectionState::Selected => super::Character::Hovered(character),
        CharacterSelectionState::None => super::Character::None,
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
        "https://glz-{}-1.{}.a.pvp.net/pregame/v1/matches/{match_id}",
        session.region.to_string(),
        session.shard.to_string()
    );

    let info = http
        .get(endpoint)
        .bearer_auth(&entitlements.token)
        .header("X-Riot-Entitlements-JWT", &entitlements.jwt)
        .send()
        .await?;

    if !info.status().is_success() {
        bail!("Pregame match check failed with status code {}.", info.status());
    }

    let info: MatchInfo = info.json().await?;
    Ok(info.into())
}

