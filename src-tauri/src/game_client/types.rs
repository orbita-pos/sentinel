use serde::{Deserialize, Serialize};

/// Active player data from /liveclientdata/activeplayer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActivePlayer {
    #[serde(rename = "riotIdGameName", default)]
    pub riot_id_game_name: String,
    #[serde(rename = "riotIdTagLine", default)]
    pub riot_id_tag_line: String,
    #[serde(default)]
    pub level: i64,
    #[serde(rename = "currentGold", default)]
    pub current_gold: f64,
    #[serde(rename = "championStats", default)]
    pub champion_stats: serde_json::Value,
    #[serde(rename = "fullRunes", default)]
    pub full_runes: serde_json::Value,
}

/// Player data from /liveclientdata/playerlist
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AllPlayer {
    #[serde(rename = "riotIdGameName", default)]
    pub riot_id_game_name: String,
    #[serde(rename = "riotIdTagLine", default)]
    pub riot_id_tag_line: String,
    #[serde(rename = "championName", default)]
    pub champion_name: String,
    #[serde(default)]
    pub team: String, // "ORDER" or "CHAOS"
    #[serde(default)]
    pub level: i64,
    #[serde(default)]
    pub items: Vec<GameItem>,
    #[serde(default)]
    pub scores: PlayerScores,
    #[serde(rename = "summonerSpells", default)]
    pub summoner_spells: serde_json::Value,
    #[serde(rename = "skinID", default)]
    pub skin_id: i64,
    #[serde(rename = "isBot", default)]
    pub is_bot: bool,
}

/// An item in a player's inventory
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameItem {
    #[serde(rename = "itemID", default)]
    pub item_id: i64,
    #[serde(rename = "displayName", default)]
    pub display_name: String,
    #[serde(default)]
    pub count: i64,
    #[serde(default)]
    pub price: i64,
}

/// Player scores from the scoreboard
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerScores {
    #[serde(default)]
    pub kills: i64,
    #[serde(default)]
    pub deaths: i64,
    #[serde(default)]
    pub assists: i64,
    #[serde(rename = "creepScore", default)]
    pub creep_score: i64,
    #[serde(rename = "wardScore", default)]
    pub ward_score: f64,
}

/// Game event from /liveclientdata/eventdata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    #[serde(rename = "EventID", default)]
    pub event_id: i64,
    #[serde(rename = "EventName", default)]
    pub event_name: String,
    #[serde(rename = "EventTime", default)]
    pub event_time: f64,
    // Other fields vary by event type, captured generically
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

/// Event list wrapper
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventData {
    #[serde(rename = "Events", default)]
    pub events: Vec<GameEvent>,
}

/// Game stats from /liveclientdata/gamestats
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameStats {
    #[serde(rename = "gameMode", default)]
    pub game_mode: String,
    #[serde(rename = "gameTime", default)]
    pub game_time: f64,
    #[serde(rename = "mapName", default)]
    pub map_name: String,
    #[serde(rename = "mapNumber", default)]
    pub map_number: i64,
    #[serde(rename = "mapTerrain", default)]
    pub map_terrain: String,
}
