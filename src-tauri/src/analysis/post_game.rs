use std::sync::Arc;

use serde::Serialize;

use crate::database::Database;
use crate::riot_api::types::{MatchTimeline, RiotMatch};

use super::patterns;

/// Post-game analysis with key moments and pattern matches
#[derive(Debug, Clone, Serialize)]
pub struct PostGameAnalysis {
    pub match_id: String,
    pub outcome: String,
    pub duration: String,
    pub champion_name: String,
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
    pub cs: i64,
    pub key_moments: Vec<KeyMoment>,
    pub pattern_matches: Vec<PatternMatch>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyMoment {
    pub timestamp: String, // "12:30"
    pub minute: i64,
    pub description: String,
    pub gold_impact: i64,
    pub category: String, // "Death", "Objective", "GoldSwing"
}

#[derive(Debug, Clone, Serialize)]
pub struct PatternMatch {
    pub pattern_id: String,
    pub pattern_description: String,
    pub evidence: String,
}

/// Generate a post-game analysis for a match
pub fn analyze(
    match_data: &RiotMatch,
    timeline: &MatchTimeline,
    puuid: &str,
    db: &Arc<Database>,
) -> Option<PostGameAnalysis> {
    let participant = match_data
        .info
        .participants
        .iter()
        .find(|p| p.puuid == puuid)?;

    let pid = participant.participant_id.to_string();
    let pid_i64 = participant.participant_id as i64;
    let team_id = participant.team_id;
    let duration_secs = match_data.info.game_duration;
    let duration_min = duration_secs / 60;
    let duration_sec = duration_secs % 60;

    // ── Find key moments (largest gold swings) ────────────

    let mut gold_by_minute: Vec<(i64, i64)> = Vec::new(); // (minute, my_gold)
    let mut team_gold_by_minute: Vec<(i64, i64, i64)> = Vec::new(); // (minute, my_team_gold, enemy_team_gold)

    for frame in &timeline.info.frames {
        let min = frame.timestamp / 60000;
        if let Some(pf) = frame.participant_frames.get(&pid) {
            gold_by_minute.push((min, pf.total_gold));
        }

        // Team gold totals
        let mut my_team_gold = 0i64;
        let mut enemy_gold = 0i64;
        for (fid, pf) in &frame.participant_frames {
            let fid_num: i64 = fid.parse().unwrap_or(0);
            // Participants 1-5 are team 100, 6-10 are team 200
            let is_my_team = (fid_num <= 5 && team_id == 100) || (fid_num > 5 && team_id == 200);
            if is_my_team {
                my_team_gold += pf.total_gold;
            } else {
                enemy_gold += pf.total_gold;
            }
        }
        team_gold_by_minute.push((min, my_team_gold, enemy_gold));
    }

    // Find the biggest gold diff swings between consecutive frames
    let mut key_moments = Vec::new();

    if team_gold_by_minute.len() >= 2 {
        let mut swings: Vec<(i64, i64)> = Vec::new(); // (minute, swing_amount)
        for i in 1..team_gold_by_minute.len() {
            let (min, my, en) = team_gold_by_minute[i];
            let (_, prev_my, prev_en) = team_gold_by_minute[i - 1];
            let prev_diff = prev_my - prev_en;
            let curr_diff = my - en;
            let swing = curr_diff - prev_diff;
            swings.push((min, swing));
        }

        // Sort by absolute swing and take top 3
        swings.sort_by(|a, b| b.1.abs().cmp(&a.1.abs()));
        for (min, swing) in swings.iter().take(3) {
            if swing.abs() < 300 {
                continue; // Ignore tiny swings
            }
            let category = if *swing < -500 { "GoldLost" } else if *swing > 500 { "GoldGained" } else { "GoldSwing" };
            let desc = if *swing < 0 {
                format!("Your team lost {} gold advantage this minute", swing.abs())
            } else {
                format!("Your team gained {} gold advantage this minute", swing)
            };
            key_moments.push(KeyMoment {
                timestamp: format!("{}:{:02}", min, 0),
                minute: *min,
                description: desc,
                gold_impact: *swing,
                category: category.to_string(),
            });
        }
    }

    // Add death moments
    for frame in &timeline.info.frames {
        for event in &frame.events {
            if event.get("type").and_then(|v| v.as_str()) == Some("CHAMPION_KILL")
                && event.get("victimId").and_then(|v| v.as_i64()) == Some(pid_i64)
            {
                let ts = event.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0);
                let min = ts / 60000;
                let sec = (ts % 60000) / 1000;
                key_moments.push(KeyMoment {
                    timestamp: format!("{min}:{sec:02}"),
                    minute: min,
                    description: format!("You were killed at {min}:{sec:02}"),
                    gold_impact: -300, // Approximate death gold
                    category: "Death".to_string(),
                });
            }
        }
    }

    // Sort by absolute impact, take top 5
    key_moments.sort_by(|a, b| b.gold_impact.abs().cmp(&a.gold_impact.abs()));
    key_moments.truncate(5);
    key_moments.sort_by_key(|m| m.minute);

    // ── Match against known patterns ──────────────────────

    let mut pattern_matches = Vec::new();
    if let Ok(stored_patterns) = db.get_patterns(puuid) {
        // Extract features for this game
        if let Some(features) = patterns::extract_features(match_data, timeline, puuid) {
            for pattern in &stored_patterns {
                let pid = pattern.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let desc = pattern.get("description").and_then(|v| v.as_str()).unwrap_or("");

                let matches = match pid {
                    "lead_throwing" => features.threw_lead,
                    "early_deaths" => features.deaths_before_15 >= 2,
                    "late_caught_out" => features.deaths_after_25 >= 3,
                    "low_cs" => features.cs_at_15.unwrap_or(100) < 90,
                    "low_vision" => features.vision_score_per_min < 0.6,
                    _ => false,
                };

                if matches {
                    pattern_matches.push(PatternMatch {
                        pattern_id: pid.to_string(),
                        pattern_description: desc.to_string(),
                        evidence: format!("This pattern appeared in this game"),
                    });
                }
            }
        }
    }

    Some(PostGameAnalysis {
        match_id: match_data.metadata.match_id.clone(),
        outcome: if participant.win { "Victory".to_string() } else { "Defeat".to_string() },
        duration: format!("{duration_min}:{duration_sec:02}"),
        champion_name: participant.champion_name.clone(),
        kills: participant.kills,
        deaths: participant.deaths,
        assists: participant.assists,
        cs: participant.total_minions_killed,
        key_moments,
        pattern_matches,
    })
}
