use std::collections::HashMap;

use color_eyre::eyre::Result;
use serde::Deserialize;

use crate::api::local::{entitlements, sessions};

use super::matchdata::Player;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct NameServiceResponse {
    /// puuid
    subject: String,
    game_name: String,
    tag_line: String,
}

/// Fetches corresponding player name for each PUUID.
///
/// * `players` - The list of players to fetch names for.
/// * `http` - The reqwest client to use for requests.
pub async fn load_player_names(
    players: &Vec<Player>,
    session_config: &sessions::Config,
    entitlements_config: &entitlements::Config,
    http: &reqwest::Client,
) -> Result<HashMap<String, String>> {
    let mut names = HashMap::new();

    let url = format!(
        "https://pd.{}.a.pvp.net/name-service/v2/players",
        session_config.shard.to_string()
    );

    let body = serde_json::to_string(
        &players
            .iter()
            .map(|p| p.puuid.clone())
            .collect::<Vec<String>>(),
    )?;

    let res = http
        .put(url)
        .bearer_auth(&entitlements_config.token)
        .header("X-Riot-Entitlements-JWT", &entitlements_config.jwt)
        .body(body)
        .send()
        .await?
        .json::<Vec<NameServiceResponse>>()
        .await?;

    debug!("Name service response: {:#?}", res);

    res.iter().for_each(|p| {
        names.insert(
            p.subject.clone(),
            format!("{} #{}", p.game_name.clone(), p.tag_line.clone()),
        );
    });

    Ok(names)
}
