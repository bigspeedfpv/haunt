use color_eyre::eyre::Result;

use crate::api::lockfile;

// the names returned by the api are confusing to say the least lol
#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    #[serde(rename = "accessToken")]
    pub token: String,
    #[serde(rename = "token")]
    pub jwt: String,
}

pub async fn login(lockfile: &lockfile::Config, http: &reqwest::Client) -> Result<Config> {
    let entitlements_endpoint = format!(
        "https://127.0.0.1:{}/entitlements/v1/token",
        lockfile.port
    );

    let res = http
        .get(&entitlements_endpoint)
        .basic_auth("riot", Some(&lockfile.password))
        .send()
        .await?
        .json::<Config>()
        .await?;

    Ok(res)
}
