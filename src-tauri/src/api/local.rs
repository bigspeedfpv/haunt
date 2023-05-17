pub mod sessions {
    use std::collections::HashMap;

    use base64::prelude::*;
    use serde::Deserialize;

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
        #[serde(rename="valorant")]
        Valorant,
        #[serde(rename="riot_client")]
        RiotClient,
    }

    #[derive(Debug)]
    pub struct SessionsConfig {
        pub shard: Shard,
        pub puuid: String,
    }

    #[derive(Debug)]
    pub enum Shard {
        Na,
        Pbe,
        Eu,
        Ap,
        Kr,
    }

    fn get_arg(arguments: &Vec<String>, prefix: &str) -> String {
        arguments
            .iter()
            .find(|&arg| arg.starts_with(prefix))
            .unwrap()
            .split_once('=')
            .unwrap()
            .1
            .to_string()
    }

    impl From<Vec<String>> for SessionsConfig {
        fn from(arguments: Vec<String>) -> Self {
            let shard = get_arg(&arguments, "-ares-deployment=");

            let puuid = get_arg(&arguments, "-subject=");

            return Self {
                shard: match shard.as_str() {
                    "na" => Shard::Na,
                    "pbe" => Shard::Pbe,
                    "eu" => Shard::Eu,
                    "ap" => Shard::Ap,
                    "kr" => Shard::Kr,
                    shard => panic!("Found shard {shard} that doesn't exist!"),
                },
                puuid: puuid.to_string(),
            };
        }
    }

    pub async fn load_config(
        lockfile_config: crate::api::lockfile::Config,
    ) -> Result<SessionsConfig, reqwest::Error> {
        // this is a local endpoint and we like never call it so :3
        let client = reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        let encoded_password =
            BASE64_STANDARD_NO_PAD.encode(format!("riot:{}", lockfile_config.password));

        let sessions_response = client
            .get(format!(
                "https://127.0.0.1:{}/product-session/v1/external-sessions",
                lockfile_config.port
            ))
            .header("Authorization", format!("Basic {}", encoded_password,))
            .send()
            .await?
            .json::<HashMap<String, SessionsResponse>>()
            .await?;

        log::info!(
            "Received reponse from sessions endpoint: {:#?}",
            sessions_response
        );

        // the API may return more than one session (e.g. for league, riot client etc)
        // so we find the one with the Valorant ID
        let valorant_config = sessions_response.values().find(|s| matches!(s.product_id, Product::Valorant)).unwrap().launch_configuration.clone();

        return Ok(SessionsConfig::from(valorant_config.arguments));
    }
}
