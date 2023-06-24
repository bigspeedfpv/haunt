use super::local::sessions;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CurrentGameResponse {
    #[serde(rename = "MatchID")]
    match_id: String,
}

pub async fn find_match_id(state: &tauri::State<'_, crate::HauntState>, session: sessions::Config) -> Option<String> {
    let mut state_handle = state.0.lock().await;
    // this function is called after both lockfile and session have succeeded
    let entitlements_config = state_handle.entitlements_config.as_ref().unwrap();

    log::debug!("Checking ingame api for player...");

    let ingame_endpoint = format!(
        "https://glz-{}-1.{}.a.pvp.net/core-game/v1/players/{}",
        &session.region.to_string(),
        &session.shard.to_string(),
        &session.puuid
    );

    let res = state_handle
        .http
        .get(&ingame_endpoint)
        .bearer_auth(&entitlements_config.token)
        .header("X-Riot-Entitlements-JWT", &entitlements_config.jwt)
        .send()
        .await;

    // Rust doesn't support if-let chaining with a bound variable in the second condition
    // a match with a guard is basically the easiest way achieve the same effect
    // we'll return early if we detect the player either in game or in a pre-game lobby
    match res {
        Ok(res) if res.status().is_success() => {
            log::debug!("Found player in game.");
            let match_id = Some(res.json::<CurrentGameResponse>().await.unwrap().match_id);

            return match_id;
        }
        _ => (),
    }

    log::debug!("Player not found. Falling back to pregame...");

    let pregame_endpoint = format!(
        "https://glz-{}-1.{}.a.pvp.net/pregame/v1/players/{}",
        &session.region.to_string(),
        &session.shard.to_string(),
        &session.puuid
    );

    let res = state_handle
        .http
        .get(&pregame_endpoint)
        .bearer_auth(&entitlements_config.token)
        .header("X-Riot-Entitlements-JWT", &entitlements_config.jwt)
        .send()
        .await;

    // there's nowhere else to check after this so we'll just assume false if they're not here
    match res {
        Ok(res) if res.status().is_success() => {
            log::debug!("Player found in pregame lobby.");
            let match_id = Some(res.json::<CurrentGameResponse>().await.unwrap().match_id);
            match_id
        }
        _ => {
            log::debug!("Player not found in a match.");
            None
        }
    }
}


