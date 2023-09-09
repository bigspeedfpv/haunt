use std::collections::HashMap;

use color_eyre::{eyre::bail, Result};
use serde::{Deserialize, Serialize};

use crate::api::valapi::seasons::Season;

use super::{entitlements, sessions};

const CLIENT_PLATFORM: &'static str = "ew0KCSJwbGF0Zm9ybVR5cGUiOiAiUEMiLA0KCSJwbGF0Zm9ybU9TIjogIldpbmRvd3MiLA0KCSJwbGF0Zm9ybU9TVmVyc2lvbiI6ICIxMC4wLjE5MDQyLjEuMjU2LjY0Yml0IiwNCgkicGxhdGZvcm1DaGlwc2V0IjogIlVua25vd24iDQp9";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PlayerMMRResponse {
    queue_skills: HashMap<String, QueueSkill>,
}

#[derive(Debug, Deserialize)]
struct QueueSkill {
    #[serde(rename = "SeasonalInfoBySeasonID")]
    seasonal_info_by_season_id: Option<HashMap<String, SeasonalInfoResponse>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SeasonalInfoResponse {
    // the fact that ID is in all caps is so incredibly annoying
    #[serde(rename = "SeasonID")]
    pub season_id: String,
    pub competitive_tier: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeasonalInfo {
    pub episode_id: String,
    pub season_id: String,
    pub competitive_tier: u32,
}

pub type History = Vec<SeasonalInfo>;

pub async fn get_player_history(
    puuid: &str,
    http: &reqwest::Client,
    entitlements: &entitlements::Config,
    session: &sessions::Config,
    acts: &[Season],
) -> Result<History> {
    let endpoint = format!(
        "https://pd.{}.a.pvp.net/mmr/v1/players/{puuid}",
        session.shard.to_string()
    );

    let mut res = http
        .get(&endpoint)
        .bearer_auth(&entitlements.token)
        .header("X-Riot-Entitlements-JWT", &entitlements.jwt)
        .header("X-Riot-ClientPlatform", CLIENT_PLATFORM)
        .header("X-Riot-ClientVersion", &session.version)
        .send()
        .await?
        .json::<PlayerMMRResponse>()
        .await?;

    // remove takes ownership of the thing!
    let mut competitive = match res.queue_skills.remove("competitive") {
        Some(competitive) => competitive.seasonal_info_by_season_id.unwrap_or(HashMap::new()),
        None => bail!("No competitive history found for player. This is probably an issue with the API lol RIP!"),
    };

    let mut history = History::new();
    for act in acts {
        if let Some(act_info) = competitive.remove(&act.season_uuid) {
            history.push(SeasonalInfo {
                episode_id: act.competitive_tiers_uuid.clone(),
                season_id: act_info.season_id,
                competitive_tier: act_info.competitive_tier,
            });
        }
    }

    debug!("Loaded history for {puuid}: {:#?}", history);

    Ok(history)
}
