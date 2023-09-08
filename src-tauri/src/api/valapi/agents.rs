use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct AgentsResponse {
    data: Vec<Agent>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub uuid: String,
    pub display_name: String,
    pub display_icon: String,
}

#[tauri::command]
pub fn load_agent_map() -> Vec<Agent> {
    let agents = reqwest::blocking::get("https://valorant-api.com/v1/agents?isPlayableCharacter=true")
        .expect("ValAPI Agents request failed")
        .json::<AgentsResponse>()
        .expect("Parse ValAPI Agents failed");

    debug!(agents = ?agents.data, "Successfully loaded agents.");

    agents.data
}
