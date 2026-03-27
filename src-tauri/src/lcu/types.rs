use serde::{Deserialize, Serialize};

/// Data parsed from the League client lockfile
#[derive(Debug, Clone)]
pub struct LockfileData {
    pub pid: u32,
    pub port: u16,
    pub password: String,
    pub protocol: String,
}

impl LockfileData {
    /// Parse lockfile content: "LeagueClient:PID:PORT:PASSWORD:PROTOCOL"
    pub fn parse(content: &str) -> Option<Self> {
        let parts: Vec<&str> = content.trim().split(':').collect();
        if parts.len() < 5 {
            return None;
        }
        Some(Self {
            pid: parts[1].parse().ok()?,
            port: parts[2].parse().ok()?,
            password: parts[3].to_string(),
            protocol: parts[4].to_string(),
        })
    }
}

/// Game flow phases from the League client
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameFlowPhase {
    None,
    Lobby,
    Matchmaking,
    ReadyCheck,
    ChampSelect,
    GameStart,
    InProgress,
    WaitingForStats,
    PreEndOfGame,
    EndOfGame,
    Reconnect,
    #[serde(other)]
    Unknown,
}

impl GameFlowPhase {
    pub fn from_str_lossy(s: &str) -> Self {
        // LCU returns phase as a quoted string like "ChampSelect"
        let s = s.trim().trim_matches('"');
        match s {
            "None" => Self::None,
            "Lobby" => Self::Lobby,
            "Matchmaking" => Self::Matchmaking,
            "ReadyCheck" => Self::ReadyCheck,
            "ChampSelect" => Self::ChampSelect,
            "GameStart" => Self::GameStart,
            "InProgress" => Self::InProgress,
            "WaitingForStats" => Self::WaitingForStats,
            "PreEndOfGame" => Self::PreEndOfGame,
            "EndOfGame" => Self::EndOfGame,
            "Reconnect" => Self::Reconnect,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Lobby => "Lobby",
            Self::Matchmaking => "Matchmaking",
            Self::ReadyCheck => "ReadyCheck",
            Self::ChampSelect => "ChampSelect",
            Self::GameStart => "GameStart",
            Self::InProgress => "InProgress",
            Self::WaitingForStats => "WaitingForStats",
            Self::PreEndOfGame => "PreEndOfGame",
            Self::EndOfGame => "EndOfGame",
            Self::Reconnect => "Reconnect",
            Self::Unknown => "Unknown",
        }
    }
}

impl std::fmt::Display for GameFlowPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Summoner data from the LCU
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LcuSummoner {
    #[serde(default)]
    pub puuid: String,
    #[serde(rename = "gameName", default)]
    pub game_name: String,
    #[serde(rename = "tagLine", default)]
    pub tag_line: String,
    #[serde(rename = "summonerId", default)]
    pub summoner_id: i64,
    #[serde(rename = "accountId", default)]
    pub account_id: i64,
    #[serde(rename = "profileIconId", default)]
    pub profile_icon_id: i64,
    #[serde(rename = "summonerLevel", default)]
    pub summoner_level: i64,
}

/// Events emitted by the LCU system
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum LcuEvent {
    Connected {
        summoner: LcuSummoner,
    },
    Disconnected,
    GameFlowChanged {
        phase: GameFlowPhase,
    },
    ChampSelectUpdate {
        data: serde_json::Value,
    },
    EndOfGame {
        data: serde_json::Value,
    },
}

/// Connection status for the frontend
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionState {
    pub status: String,
    pub summoner: Option<LcuSummoner>,
    pub game_phase: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lockfile() {
        let content = "LeagueClient:12345:8443:abc123def:https";
        let data = LockfileData::parse(content).unwrap();
        assert_eq!(data.pid, 12345);
        assert_eq!(data.port, 8443);
        assert_eq!(data.password, "abc123def");
        assert_eq!(data.protocol, "https");
    }

    #[test]
    fn test_parse_lockfile_invalid() {
        assert!(LockfileData::parse("invalid").is_none());
        assert!(LockfileData::parse("a:b:c").is_none());
    }

    #[test]
    fn test_gameflow_from_str() {
        assert_eq!(GameFlowPhase::from_str_lossy("ChampSelect"), GameFlowPhase::ChampSelect);
        assert_eq!(GameFlowPhase::from_str_lossy("\"InProgress\""), GameFlowPhase::InProgress);
        assert_eq!(GameFlowPhase::from_str_lossy("garbage"), GameFlowPhase::Unknown);
    }
}
