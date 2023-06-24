use color_eyre::Result;
use serde::Deserialize;

use crate::api::local::sessions;

#[derive(Deserialize)]
struct SeasonsResponse {
    status: i32,
    data: Vec<Season>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Season {
    pub uuid: String,
    season_uuid: String,
    competitive_tiers_uuid: String,
    borders: Option<Vec<Border>>,
    asset_path: String,
    start_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Border {
    uuid: String,
    level: i32,
    wins_required: i32,
    display_icon: String,
    small_icon: Option<String>,
}

pub async fn get_prev_3_seasons(
    state: &tauri::State<'_, crate::HauntState>,
) -> Result<Vec<Season>> {
    let seasons = state
        .0
        .http
        .get("https://valorant-api.com/v1/seasons/competitive")
        .send()
        .await?
        .json::<SeasonsResponse>()
        .await?;

    // seasons response includes both episodes and acts - we only want acts
    // also ensure it only includes past acts
    let mut seasons = seasons
        .data
        .into_iter()
        .filter(|s| s.asset_path.contains("Act") && s.start_time < chrono::Utc::now())
        .collect::<Vec<_>>();

    // sort by start date, descending (most recent first)
    seasons.sort_by(|a, b| b.start_time.cmp(&a.start_time));

    Ok(seasons)
}
