use serde::{Deserialize, Serialize};

/// Parsed champion select session from the LCU
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChampSelectSession {
    pub my_team: Vec<ChampSelectPlayer>,
    pub their_team: Vec<ChampSelectPlayer>,
    pub bans: Vec<i64>,
    pub local_player_cell_id: i64,
    pub phase: String,
    pub timer_remaining: f64,
}

/// A player in champion select
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChampSelectPlayer {
    pub cell_id: i64,
    pub champion_id: i64,
    pub summoner_id: i64,
    pub assigned_position: String,
    pub is_local_player: bool,
    /// Populated by scouting (Phase 4)
    pub game_name: Option<String>,
    pub tag_line: Option<String>,
    pub puuid: Option<String>,
    pub rank: Option<String>,
}

/// Parse raw LCU champ select session JSON into our typed structure
pub fn parse_session(data: &serde_json::Value, local_cell_id_override: Option<i64>) -> Option<ChampSelectSession> {
    let obj = data.as_object()?;

    let local_player_cell_id = local_cell_id_override
        .or_else(|| obj.get("localPlayerCellId").and_then(|v| v.as_i64()))
        .unwrap_or(-1);

    // Parse my team
    let my_team = parse_team(
        obj.get("myTeam").and_then(|v| v.as_array()),
        local_player_cell_id,
    );

    // Parse their team (may be empty in blind pick)
    let their_team = parse_team(
        obj.get("theirTeam").and_then(|v| v.as_array()),
        -1, // no local player on enemy team
    );

    // Parse bans from actions
    let bans = parse_bans(obj.get("actions").and_then(|v| v.as_array()));

    // Parse timer
    let timer = obj.get("timer").and_then(|t| t.as_object());
    let timer_remaining = timer
        .and_then(|t| t.get("adjustedTimeLeftInPhase"))
        .or_else(|| timer.and_then(|t| t.get("timeLeftInPhase")))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
        / 1000.0; // ms to seconds

    let phase = timer
        .and_then(|t| t.get("phase"))
        .and_then(|v| v.as_str())
        .unwrap_or("UNKNOWN")
        .to_string();

    Some(ChampSelectSession {
        my_team,
        their_team,
        bans,
        local_player_cell_id,
        phase,
        timer_remaining,
    })
}

fn parse_team(team_arr: Option<&Vec<serde_json::Value>>, local_cell_id: i64) -> Vec<ChampSelectPlayer> {
    let Some(arr) = team_arr else { return vec![] };

    arr.iter()
        .filter_map(|p| {
            let cell_id = p.get("cellId").and_then(|v| v.as_i64()).unwrap_or(-1);
            let champion_id = p.get("championId")
                .or_else(|| p.get("championPickIntent"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let summoner_id = p.get("summonerId").and_then(|v| v.as_i64()).unwrap_or(0);
            let assigned_position = p.get("assignedPosition")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_uppercase();

            Some(ChampSelectPlayer {
                cell_id,
                champion_id,
                summoner_id,
                assigned_position,
                is_local_player: cell_id == local_cell_id,
                game_name: None,
                tag_line: None,
                puuid: None,
                rank: None,
            })
        })
        .collect()
}

fn parse_bans(actions: Option<&Vec<serde_json::Value>>) -> Vec<i64> {
    let Some(action_groups) = actions else { return vec![] };
    let mut bans = Vec::new();

    for group in action_groups {
        let Some(group_arr) = group.as_array() else { continue };
        for action in group_arr {
            let is_ban = action.get("type").and_then(|v| v.as_str()) == Some("ban");
            let completed = action.get("completed").and_then(|v| v.as_bool()).unwrap_or(false);
            if is_ban && completed {
                if let Some(champ_id) = action.get("championId").and_then(|v| v.as_i64()) {
                    if champ_id > 0 {
                        bans.push(champ_id);
                    }
                }
            }
        }
    }
    bans
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_session_basic() {
        let data = json!({
            "localPlayerCellId": 0,
            "myTeam": [
                {"cellId": 0, "championId": 39, "summonerId": 123, "assignedPosition": "top"},
                {"cellId": 1, "championId": 0, "summonerId": 456, "assignedPosition": "jungle"}
            ],
            "theirTeam": [
                {"cellId": 5, "championId": 86, "summonerId": 789, "assignedPosition": "top"}
            ],
            "actions": [
                [{"type": "ban", "completed": true, "championId": 157}],
                [{"type": "ban", "completed": true, "championId": 238}]
            ],
            "timer": {
                "adjustedTimeLeftInPhase": 25000,
                "phase": "BAN_PICK"
            }
        });

        let session = parse_session(&data, None).unwrap();
        assert_eq!(session.my_team.len(), 2);
        assert_eq!(session.their_team.len(), 1);
        assert_eq!(session.my_team[0].champion_id, 39);
        assert!(session.my_team[0].is_local_player);
        assert!(!session.my_team[1].is_local_player);
        assert_eq!(session.bans, vec![157, 238]);
        assert_eq!(session.phase, "BAN_PICK");
        assert!((session.timer_remaining - 25.0).abs() < 0.1);
    }

    #[test]
    fn test_parse_empty_session() {
        let data = json!({});
        let session = parse_session(&data, None);
        assert!(session.is_some()); // Should handle missing fields gracefully
        let s = session.unwrap();
        assert!(s.my_team.is_empty());
        assert!(s.bans.is_empty());
    }
}
