use color_eyre::eyre::Result;

// the names returned by the api are confusing to say the least lol
#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    #[serde(rename = "accessToken")]
    pub token: String,
    #[serde(rename = "token")]
    pub jwt: String,
}

pub async fn login(state: &tauri::State<'_, crate::HauntState>) -> Result<Config> {
    let state_handle = state.0.lock().await;
    let lockfile_config = state_handle.lockfile_config.as_ref().unwrap();

    let http = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let entitlements_endpoint = format!(
        "https://127.0.0.1:{}/entitlements/v1/token",
        lockfile_config.port
    );

    let res = http
        .get(&entitlements_endpoint)
        .basic_auth("riot", Some(&lockfile_config.password))
        .send()
        .await?
        .json::<Config>()
        .await?;

    Ok(res)
}
