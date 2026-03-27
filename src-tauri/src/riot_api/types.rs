use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Match V5 ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiotMatch {
    pub metadata: MatchMetadata,
    pub info: MatchInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchMetadata {
    #[serde(rename = "matchId")]
    pub match_id: String,
    pub participants: Vec<String>, // PUUIDs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchInfo {
    #[serde(rename = "gameCreation")]
    pub game_creation: i64,
    #[serde(rename = "gameDuration")]
    pub game_duration: i64,
    #[serde(rename = "gameMode")]
    pub game_mode: String,
    #[serde(rename = "gameVersion", default)]
    pub game_version: String,
    #[serde(rename = "queueId")]
    pub queue_id: i64,
    pub participants: Vec<Participant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub puuid: String,
    #[serde(rename = "participantId")]
    pub participant_id: i32,
    #[serde(rename = "championId")]
    pub champion_id: i64,
    #[serde(rename = "championName")]
    pub champion_name: String,
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
    #[serde(rename = "totalMinionsKilled")]
    pub total_minions_killed: i64,
    #[serde(rename = "goldEarned")]
    pub gold_earned: i64,
    #[serde(rename = "totalDamageDealtToChampions", default)]
    pub total_damage_dealt: i64,
    #[serde(rename = "visionScore", default)]
    pub vision_score: i64,
    pub win: bool,
    #[serde(rename = "teamPosition", default)]
    pub team_position: String,
    #[serde(rename = "teamId")]
    pub team_id: i64,
    #[serde(rename = "summonerName", default)]
    pub summoner_name: String,
    #[serde(rename = "riotIdGameName", default)]
    pub riot_id_game_name: String,
    #[serde(rename = "riotIdTagline", default)]
    pub riot_id_tagline: String,
}

// ── Timeline V5 ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchTimeline {
    pub metadata: MatchMetadata,
    pub info: TimelineInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineInfo {
    #[serde(rename = "frameInterval")]
    pub frame_interval: i64,
    pub frames: Vec<TimelineFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineFrame {
    pub timestamp: i64,
    #[serde(rename = "participantFrames")]
    pub participant_frames: HashMap<String, ParticipantFrame>,
    #[serde(default)]
    pub events: Vec<serde_json::Value>, // Keep events as raw JSON for flexibility
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantFrame {
    #[serde(default)]
    pub position: Option<Position>,
    #[serde(rename = "totalGold", default)]
    pub total_gold: i64,
    #[serde(rename = "currentGold", default)]
    pub current_gold: i64,
    #[serde(default)]
    pub xp: i64,
    #[serde(rename = "minionsKilled", default)]
    pub minions_killed: i64,
    #[serde(rename = "jungleMinionsKilled", default)]
    pub jungle_minions_killed: i64,
    #[serde(default)]
    pub level: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

// ── League / Ranked ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeagueEntry {
    #[serde(rename = "queueType")]
    pub queue_type: String,
    pub tier: Option<String>,
    pub rank: Option<String>,
    #[serde(rename = "leaguePoints", default)]
    pub league_points: i32,
    #[serde(default)]
    pub wins: i32,
    #[serde(default)]
    pub losses: i32,
}

// ── Region mapping ────────────────────────────────────────

/// Map a platform (na1, euw1, etc.) to the regional routing value (americas, europe, asia)
pub fn platform_to_region(platform: &str) -> &'static str {
    match platform {
        "na1" | "br1" | "la1" | "la2" => "americas",
        "euw1" | "eun1" | "tr1" | "ru" => "europe",
        "kr" | "jp1" => "asia",
        "oc1" | "ph2" | "sg2" | "th2" | "tw2" | "vn2" => "sea",
        _ => "americas",
    }
}
