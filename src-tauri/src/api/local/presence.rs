#![allow(unused)]
use std::path::Path;

use base64::prelude::*;
use color_eyre::eyre::Result;
use heck::ToTitleCase;
use serde::{Deserialize, Serialize};

use crate::api::lockfile;

const DM_UUID: &'static str = "a8790ec5-4237-f2f0-e93b-08a8e89865b2";
const SPIKE_RUSH_UUID: &'static str = "e921d1e6-416b-c31f-1291-74930c330b7b";
const GGTEAM_UUID: &'static str = "a4ed6518-4741-6dcb-35bd-f884aecdc859";
const ONEFA_UUID: &'static str = "96bd3920-4f36-d026-2b28-c683eb0bcac5";
const SNOWBALL_UUID: &'static str = "57038d6d-49b1-3a74-c5ef-3395d9f23a97";

#[derive(Debug, Deserialize)]
struct PresenceResponse {
    presences: Vec<Presence>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Product {
    Valorant,
    LeagueOfLegends,
}

#[derive(Debug, Deserialize)]
struct Presence {
    puuid: String,
    game_name: String,
    game_tag: String,
    product: Product,
    private: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Player {
    pub puuid: String,
    pub game_name: String,
    pub game_tag: String,
    pub private: Private,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Private {
    pub is_valid: bool,
    pub match_map: String,
    pub party_id: String,
    pub player_card_id: String,
    pub player_title_id: String,
    pub preferred_level_border_id: String,
    pub account_level: u32,
    pub competitive_tier: u32,
    pub queue_id: String, // gamemode id!
}

// converts the raw presence response to a Player object with a puuid and decoded Private presence
impl From<Presence> for Player {
    fn from(value: Presence) -> Self {
        // private data is base64 encoded json
        // this only ever gets called on ingame players with private presences!
        let decoded = BASE64_STANDARD.decode(value.private.unwrap()).unwrap();
        // decode returns a Vec of chars so we need to convert to a String
        let decoded = String::from_utf8(decoded).unwrap();
        let private = serde_json::from_str(&decoded).unwrap();

        Player {
            puuid: value.puuid,
            game_name: value.game_name,
            game_tag: value.game_tag,
            private,
        }
    }
}

pub async fn get_match_players(lockfile: &lockfile::Config, http: &reqwest::Client) -> Result<Vec<Player>> {
    let presences_endpoint = format!(
        "https://127.0.0.1:{}/chat/v4/presences",
        &lockfile.port
    );

    let presences = http
        .get(presences_endpoint)
        .basic_auth("riot", Some(&lockfile.password))
        .send()
        .await?
        .json::<PresenceResponse>()
        .await?
        .presences;

    // we'll only collect players playing val with private statuses
    // into_iter consumes the original value and owns its values rather than providing references
    // we also augment presences to Players to make it easier to work with later
    let players = presences
        .into_iter()
        .filter(|p| p.product == Product::Valorant && p.private.is_some())
        .map(|p| {
            log::debug!("Found presence: {:#?}", p);
            Player::from(p)
        })
        .collect();

    Ok(players)
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchInfo {
    gamemode: String,
    map: Map,
}

#[derive(Debug, Clone, Serialize)]
pub enum Map {
    Bind,
    Haven,
    Split,
    Ascent,
    Icebox,
    Breeze,
    Fracture,
    TheRange,
    Unknown,
}

impl From<&Player> for MatchInfo {
    fn from(player: &Player) -> Self {
        let map = Map::from(player.private.match_map.clone());

        let gamemode_name = match map {
            Map::TheRange => String::from("The Range"),
            _ => match player.private.queue_id.as_str() {
                "competitive" => String::from("Competitive"),
                "unrated" => String::from("Unrated"),
                "newmap" => String::from("New Map"),
                // "deathmatch" => get_map_name(DM_UUID),
                // "spikerush" => get_map_name(SPIKE_RUSH_UUID),
                // "ggteam" => get_map_name(GGTEAM_UUID),
                // "onefa" => get_map_name(ONEFA_UUID),
                // "snowball" => get_map_name(SNOWBALL_UUID),
                m => m.to_title_case(),
            },
        };

        Self {
            gamemode: gamemode_name,
            map: Map::from(player.private.match_map.clone()),
        }
    }
}

impl From<String> for Map {
    fn from(value: String) -> Self {
        let value = value.split('/').last().unwrap();
        match value {
            "Bind" => Self::Bind,
            "Haven" => Self::Haven,
            "Split" => Self::Split,
            "Ascent" => Self::Ascent,
            "Icebox" => Self::Icebox,
            "Breeze" => Self::Breeze,
            "Fracture" => Self::Fracture,
            "Range" => Self::TheRange,
            _ => Self::Unknown,
        }
    }
}

fn get_map_name(uuid: &'static str) -> String {
    let maps_path = Path::new(&std::env::var("LOCALAPPDATA").expect("No LocalAppData var!"))
        .join("Haunt")
        .join("ValAPI")
        .join("maps.txt");

    String::from("")
}

pub async fn get_match_info(puuid: String, players: &Vec<Player>) -> MatchInfo {
    MatchInfo::from(players.iter().find(|p| p.puuid == puuid).unwrap())
}
