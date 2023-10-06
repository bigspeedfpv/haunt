use std::collections::HashMap;

use color_eyre::eyre::{eyre, Result};
use serde::Deserialize;

use crate::api::lockfile;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionsResponse {
    launch_configuration: LaunchConfiguration,
    product_id: Product,
}

#[derive(Clone, Debug, Deserialize)]
struct LaunchConfiguration {
    arguments: Vec<String>,
}

#[derive(Debug, Deserialize)]
enum Product {
    #[serde(rename = "valorant")]
    Valorant,
    #[serde(rename = "riot_client")]
    RiotClient,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub puuid: String,
    pub region: Region,
    pub shard: Shard,
    pub version: String,
}

#[derive(Clone, Debug)]
pub enum Region {
    Na,
    Latam,
    Br,
    Eu,
    Ap,
    Kr,
}

impl From<&String> for Region {
    fn from(value: &String) -> Self {
        match value.as_str() {
            "latam" => Region::Latam,
            "br" => Region::Br,
            "eu" => Region::Eu,
            "ap" => Region::Ap,
            "kr" => Region::Kr,
            _ => Region::Na,
        }
    }
}

impl std::string::ToString for Region {
    fn to_string(&self) -> String {
        match self {
            Region::Na => "na".to_string(),
            Region::Latam => "latam".to_string(),
            Region::Br => "br".to_string(),
            Region::Eu => "eu".to_string(),
            Region::Ap => "ap".to_string(),
            Region::Kr => "kr".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Shard {
    Na,
    Pbe,
    Eu,
    Ap,
    Kr,
}

impl From<&String> for Shard {
    fn from(value: &String) -> Self {
        match value.as_str() {
            "pbe" => Shard::Pbe,
            "eu" => Shard::Eu,
            "ap" => Shard::Ap,
            "kr" => Shard::Kr,
            _ => Shard::Na,
        }
    }
}

impl std::string::ToString for Shard {
    fn to_string(&self) -> String {
        match self {
            Shard::Na => "na".to_string(),
            Shard::Pbe => "pbe".to_string(),
            Shard::Eu => "eu".to_string(),
            Shard::Ap => "ap".to_string(),
            Shard::Kr => "kr".to_string(),
        }
    }
}

fn get_arg(arguments: &Vec<String>, prefix: &str) -> String {
    arguments
        .iter()
        .find(|&arg| arg.starts_with(prefix))
        .unwrap()
        .split(['=', '&']) // this is taken from WAIUA - & possibly used for diff regions?
        .nth(1)
        .unwrap()
        .to_string()
}

impl From<&SessionsResponse> for Config {
    fn from(config: &SessionsResponse) -> Self {
        let arguments = config.launch_configuration.arguments.clone();

        let deployment = get_arg(&arguments, "-ares-deployment=");
        let puuid = get_arg(&arguments, "-subject=");

        return Self {
            puuid: puuid.to_string(),
            shard: Shard::from(&deployment),
            region: Region::from(&deployment),
            version: "".to_string(),
        };
    }
}

#[derive(Deserialize)]
struct VersionResponse {
    data: VersionData,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionData {
    riot_client_version: String,
}

/// Loads the current Session, including the Shard, Region, and Version.
/// * `lockfile` - Config loaded from the lockfile.
/// * `offline_http` - HTTP client used for offline requests.
/// * `http` - HTTPS client used for online requests.
pub async fn load_config(
    lockfile: &lockfile::Config,
    offline_http: &reqwest::Client,
    http: &reqwest::Client,
) -> Result<Config> {
    let sessions_response = offline_http
        .get(format!(
            "https://127.0.0.1:{}/product-session/v1/external-sessions",
            lockfile.port
        ))
        .basic_auth("riot", Some(&lockfile.password)) // this b64 encodes for us omg!
        .send()
        .await?
        .json::<HashMap<String, SessionsResponse>>()
        .await?;

    let version = http
        .get("https://valorant-api.com/v1/version")
        .send()
        .await?
        .json::<VersionResponse>()
        .await?;

    // the API may return more than one session (e.g. for league, riot client etc)
    // so we find the one with the Valorant ID
    let valorant_config = sessions_response
        .values()
        .find(|s| matches!(s.product_id, Product::Valorant))
        .ok_or_else(|| eyre!("Failed to find Valorant session"))?;

    let mut valorant_config = Config::from(valorant_config);
    valorant_config.version = version.data.riot_client_version;

    return Ok(Config::from(valorant_config));
}
