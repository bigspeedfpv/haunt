#![allow(unused)]
pub struct Url {
    pub name: &'static str,
    pub suffix: &'static str,
    pub url: &'static str,
}

pub const BASE_URL: &str = "https://valorant-api.com/v1";

pub const MAPS: Url = Url {
    name: "Maps",
    suffix: "maps.txt",
    url: "/maps",
};

pub const AGENTS: Url = Url {
    name: "Agents",
    suffix: "agents.txt",
    url: "/agents",
};

pub const SKINS: Url = Url {
    name: "Skins",
    suffix: "skinchromas.txt",
    url: "/weapons/skinchromas",
};

pub const CARDS: Url = Url {
    name: "Cards",
    suffix: "cards.txt",
    url: "/playercards",
};

pub const SPRAYS: Url = Url {
    name: "Sprays",
    suffix: "sprays.txt",
    url: "/sprays",
};

pub const RANKS: Url = Url {
    name: "Ranks",
    suffix: "ranks.txt",
    url: "/competitivetiers",
};

pub const VERSION: Url = Url {
    name: "Version",
    suffix: "version.txt",
    url: "/version",
};

pub const GAMEMODES: Url = Url {
    name: "Gamemodes",
    suffix: "gamemodes.txt",
    url: "/gamemodes",
};
