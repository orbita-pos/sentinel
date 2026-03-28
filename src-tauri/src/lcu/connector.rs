use super::tls;
use super::types::{GameFlowPhase, LcuSummoner, LockfileData};
use anyhow::{Context, Result};

use crate::database::MatchParticipantRow;

/// REST client for the League Client Update API
#[derive(Clone)]
pub struct LcuClient {
    client: reqwest::Client,
    base_url: String,
}

impl LcuClient {
    /// Create a new LCU client from lockfile credentials
    pub fn new(lockfile: &LockfileData) -> Self {
        let client = tls::build_lcu_http_client(&lockfile.password);
        let base_url = format!("https://127.0.0.1:{}", lockfile.port);
        Self { client, base_url }
    }

    /// Make a GET request to an LCU endpoint
    async fn get<T: serde::de::DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, endpoint);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("LCU request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("LCU returned {status}: {body}");
        }

        resp.json::<T>().await.context("Failed to parse LCU response")
    }

    /// Get raw JSON from an LCU endpoint
    async fn get_raw(&self, endpoint: &str) -> Result<String> {
        let url = format!("{}{}", self.base_url, endpoint);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("LCU request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("LCU returned {status}: {body}");
        }

        resp.text().await.context("Failed to read LCU response")
    }

    /// Get the currently logged-in summoner
    pub async fn get_current_summoner(&self) -> Result<LcuSummoner> {
        self.get("/lol-summoner/v1/current-summoner").await
    }

    /// Get the current game flow phase
    pub async fn get_gameflow_phase(&self) -> Result<GameFlowPhase> {
        let phase_str: String = self.get("/lol-gameflow/v1/gameflow-phase").await?;
        Ok(GameFlowPhase::from_str_lossy(&phase_str))
    }

    /// Get recent match history from the local client (NO API key needed)
    /// Returns parsed match data ready for DB storage
    pub async fn get_match_history(&self, puuid: &str, count: i32) -> Result<Vec<LcuMatchData>> {
        let raw = self
            .get_raw(&format!(
                "/lol-match-history/v1/products/lol/{puuid}/matches?begIndex=0&endIndex={count}"
            ))
            .await?;

        let data: serde_json::Value = serde_json::from_str(&raw)?;
        let games = data
            .get("games")
            .or_else(|| data.get("matches"))
            .and_then(|g| g.get("games").or(Some(g)))
            .and_then(|g| g.as_array())
            .cloned()
            .unwrap_or_default();

        let mut results = Vec::new();
        for game in &games {
            if let Some(parsed) = parse_lcu_match(game, puuid) {
                results.push(parsed);
            }
        }

        Ok(results)
    }

    /// Get current ranked stats from the local client
    pub async fn get_ranked_stats(&self) -> Result<serde_json::Value> {
        self.get("/lol-ranked/v1/current-ranked-stats").await
    }
}

/// Parsed match data from LCU match history
pub struct LcuMatchData {
    pub game_id: i64,
    pub match_id: String,
    pub game_creation: i64,
    pub game_duration: i64,
    pub game_mode: String,
    pub queue_id: i64,
    pub raw_json: String,
    pub participants: Vec<MatchParticipantRow>,
}

/// Parse a single LCU match history game entry
fn parse_lcu_match(game: &serde_json::Value, my_puuid: &str) -> Option<LcuMatchData> {
    let game_id = game.get("gameId").and_then(|v| v.as_i64())?;
    let game_creation = game.get("gameCreation").and_then(|v| v.as_i64()).unwrap_or(0);
    let game_duration = game.get("gameDuration").and_then(|v| v.as_i64()).unwrap_or(0);
    let game_mode = game.get("gameMode").and_then(|v| v.as_str()).unwrap_or("CLASSIC").to_string();
    let queue_id = game.get("queueId").and_then(|v| v.as_i64()).unwrap_or(0);

    // The LCU uses platform-specific match IDs like "LA1_12345"
    // Construct a match_id similar to Riot API format
    let platform = game
        .get("platformId")
        .and_then(|v| v.as_str())
        .unwrap_or("LA1");
    let match_id = format!("{platform}_{game_id}");

    // Parse participants
    let participants_arr = game
        .get("participants")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // LCU also has "participantIdentities" that maps participant IDs to PUUIDs
    let identities = game
        .get("participantIdentities")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // Build a map of participantId -> puuid
    let mut id_to_puuid: std::collections::HashMap<i64, String> = std::collections::HashMap::new();
    for identity in &identities {
        let pid = identity.get("participantId").and_then(|v| v.as_i64()).unwrap_or(0);
        let puuid_val = identity
            .get("player")
            .and_then(|p| p.get("puuid"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if !puuid_val.is_empty() {
            id_to_puuid.insert(pid, puuid_val);
        }
    }

    // Determine winning team
    let teams = game.get("teams").and_then(|v| v.as_array());
    let mut winning_team_id: i64 = 0;
    if let Some(teams) = teams {
        for team in teams {
            let win = team.get("win").and_then(|v| v.as_str()).unwrap_or("");
            if win == "Win" {
                winning_team_id = team.get("teamId").and_then(|v| v.as_i64()).unwrap_or(0);
            }
        }
    }

    let mut participant_rows = Vec::new();
    for p in &participants_arr {
        let pid = p.get("participantId").and_then(|v| v.as_i64()).unwrap_or(0);
        let stats = p.get("stats").unwrap_or(p);
        let team_id = p.get("teamId").and_then(|v| v.as_i64()).unwrap_or(0);
        let champion_id = p.get("championId").and_then(|v| v.as_i64()).unwrap_or(0);

        let puuid = id_to_puuid.get(&pid).cloned().unwrap_or_default();
        let win_val = stats.get("win").and_then(|v| v.as_bool())
            .unwrap_or(team_id == winning_team_id && winning_team_id > 0);

        let timeline = p.get("timeline");
        let role = timeline
            .and_then(|t| t.get("lane"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let position = match role {
            "TOP" => "TOP",
            "JUNGLE" => "JUNGLE",
            "MIDDLE" | "MID" => "MIDDLE",
            "BOTTOM" | "BOT" => "BOTTOM",
            _ => {
                // Try the role field
                let r = timeline.and_then(|t| t.get("role")).and_then(|v| v.as_str()).unwrap_or("");
                if r == "DUO_SUPPORT" || r == "SUPPORT" { "UTILITY" } else { role }
            }
        };

        participant_rows.push(MatchParticipantRow {
            participant_id: pid as i32,
            puuid,
            champion_id,
            champion_name: format!("Champion{champion_id}"), // Will resolve from Data Dragon
            team_id,
            team_position: position.to_string(),
            kills: stats.get("kills").and_then(|v| v.as_i64()).unwrap_or(0),
            deaths: stats.get("deaths").and_then(|v| v.as_i64()).unwrap_or(0),
            assists: stats.get("assists").and_then(|v| v.as_i64()).unwrap_or(0),
            total_minions_killed: stats.get("totalMinionsKilled").and_then(|v| v.as_i64()).unwrap_or(0)
                + stats.get("neutralMinionsKilled").and_then(|v| v.as_i64()).unwrap_or(0),
            gold_earned: stats.get("goldEarned").and_then(|v| v.as_i64()).unwrap_or(0),
            total_damage_dealt: stats.get("totalDamageDealtToChampions").and_then(|v| v.as_i64()).unwrap_or(0),
            vision_score: stats.get("visionScore").and_then(|v| v.as_i64()).unwrap_or(0),
            win: win_val,
        });
    }

    // If we don't have puuid in identities, check if this is a newer format
    // where the local player data is directly available
    if participant_rows.iter().all(|p| p.puuid.is_empty()) {
        // Mark local player by matching summoner data
        // The caller's puuid won't match, but we still store the data
    }

    Some(LcuMatchData {
        game_id,
        match_id,
        game_creation,
        game_duration,
        game_mode,
        queue_id,
        raw_json: game.to_string(),
        participants: participant_rows,
    })
}
