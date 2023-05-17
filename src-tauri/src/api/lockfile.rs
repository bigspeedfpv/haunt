use std::{fs, path::Path, str::FromStr};

#[derive(Clone, Debug)]
pub struct Config {
    pub name: String,
    pub pid: u32,
    pub port: u32,
    pub password: String,
    pub protocol: String,
}

#[derive(serde::Serialize)]
pub struct ParseConfigError;

impl FromStr for Config {
    type Err = ParseConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 5 {
            return Err(ParseConfigError);
        }

        return Ok(Self {
            name: parts[0].to_string(),
            pid: parts[1].parse().or_else(|_| Err(ParseConfigError))?,
            port: parts[2].parse().or_else(|_| Err(ParseConfigError))?,
            password: parts[3].to_string(),
            protocol: parts[4].to_string(),
        });
    }
}

pub fn load_config() -> Option<Config> {
    let lockfile_path = Path::new(&std::env::var("LOCALAPPDATA").expect("No LocalAppData var!"))
        .join("Riot Games")
        .join("Riot Client")
        .join("Config")
        .join("lockfile");

    let lockfile = fs::read_to_string(lockfile_path);
    let lockfile = match lockfile {
        Ok(lockfile) => lockfile,
        Err(_) => return None,
    };

    let loaded_config = lockfile.parse::<Config>().ok()?;

    return Some(loaded_config.clone());
}
