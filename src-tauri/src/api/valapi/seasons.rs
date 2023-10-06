use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::api::local::sessions;

#[derive(Deserialize)]
struct SeasonsResponse {
    status: i32,
    data: Vec<Season>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Season {
    pub season_uuid: String,
    pub competitive_tiers_uuid: String,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompetitiveTierResponse {
    status: i32,
    data: Vec<CompetitiveTierResponseData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompetitiveTierResponseData {
    /// act uuid
    uuid: String,
    tiers: Vec<CompetitiveTierResponseTier>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompetitiveTierResponseTier {
    /// rank index
    tier: u32,
    tier_name: String,
    small_icon: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetitiveTier {
    /// episode UUID
    pub episode: String,
    /// rank index
    pub tier: u32,
    pub tier_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

impl Into<Vec<CompetitiveTier>> for CompetitiveTierResponse {
    fn into(self) -> Vec<CompetitiveTier> {
        self.data
            .into_iter()
            .flat_map(|episode| {
                episode.tiers.into_iter().map(move |tier| CompetitiveTier {
                    episode: episode.uuid.clone(),
                    tier: tier.tier,
                    tier_name: tier.tier_name,
                    icon: tier.small_icon,
                })
            })
            .collect()
    }
}

impl Default for CompetitiveTier {
    fn default() -> Self {
        CompetitiveTier {
            episode: String::new(),
            tier: 0,
            tier_name: String::from("UNRANKED"),
            icon: None,
        }
    }
}

impl CompetitiveTier {
    /// Get the competitive tier for a given act and tier index
    ///
    /// * `tiers` - list of competitive tiers
    /// * `episode` - episode uuid
    /// * `tier` - rank index
    pub fn from_act_tier(tiers: &Vec<Self>, episode: &str, tier: u32) -> Self {
        tiers
            .iter()
            .find(|t| t.episode == episode && t.tier == tier)
            .cloned()
            .unwrap_or_default()
    }
}

pub fn load_competitive_tiers() -> Vec<CompetitiveTier> {
    let res = reqwest::blocking::get("https://valorant-api.com/v1/competitivetiers")
        .expect("Failed to get competitive tiers")
        .json::<CompetitiveTierResponse>()
        .expect("Failed to parse competitive tier response");

    let competitive_tiers = res.into();
    debug!("Loaded competitive tiers: {:#?}", competitive_tiers);

    competitive_tiers
}
