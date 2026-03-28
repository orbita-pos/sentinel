use std::sync::Arc;

use serde::Serialize;

use crate::database::Database;
use crate::riot_api::types::{MatchTimeline, RiotMatch};

/// Extracted features from a single game's timeline
#[derive(Debug, Clone, Serialize)]
pub struct GameFeatures {
    pub match_id: String,
    pub champion_id: i64,
    pub champion_name: String,
    pub role: String,
    pub win: bool,
    pub game_duration_min: f64,

    // Lane phase (0-15 min)
    pub cs_at_10: Option<i64>,
    pub cs_at_15: Option<i64>,
    pub gold_diff_at_10: Option<i64>,
    pub gold_diff_at_15: Option<i64>,
    pub deaths_before_15: i32,

    // Mid/late game
    pub gold_diff_at_20: Option<i64>,
    pub deaths_after_25: i32,

    // Cross-phase
    pub vision_score_per_min: f64,
    pub kill_participation: f64,

    // Derived
    pub had_early_lead: bool,
    pub threw_lead: bool,
}

/// Extract features from a match + timeline for a specific player
pub fn extract_features(
    match_data: &RiotMatch,
    timeline: &MatchTimeline,
    puuid: &str,
) -> Option<GameFeatures> {
    // Find the player's participant ID
    let participant = match_data
        .info
        .participants
        .iter()
        .find(|p| p.puuid == puuid)?;

    let participant_id = participant.participant_id.to_string();
    let team_id = participant.team_id;
    let game_duration_min = match_data.info.game_duration as f64 / 60.0;

    if game_duration_min < 5.0 {
        return None; // Skip remakes
    }

    // Find opponent laner (same position, different team)
    let opponent = match_data.info.participants.iter().find(|p| {
        p.team_id != team_id
            && !p.team_position.is_empty()
            && p.team_position == participant.team_position
    });
    let opponent_id = opponent.map(|o| o.participant_id.to_string());

    // Extract per-minute data from timeline frames
    let mut cs_at_10 = None;
    let mut cs_at_15 = None;
    let mut gold_at_10 = None;
    let mut gold_at_15 = None;
    let mut gold_at_20 = None;
    let mut opp_gold_at_10 = None;
    let mut opp_gold_at_15 = None;
    let mut opp_gold_at_20 = None;

    for frame in &timeline.info.frames {
        let min = frame.timestamp / 60000;
        if let Some(pf) = frame.participant_frames.get(&participant_id) {
            let cs = pf.minions_killed + pf.jungle_minions_killed;
            match min {
                10 => {
                    cs_at_10 = Some(cs);
                    gold_at_10 = Some(pf.total_gold);
                }
                15 => {
                    cs_at_15 = Some(cs);
                    gold_at_15 = Some(pf.total_gold);
                }
                20 => {
                    gold_at_20 = Some(pf.total_gold);
                }
                _ => {}
            }
        }
        if let Some(opp_id) = &opponent_id {
            if let Some(opf) = frame.participant_frames.get(opp_id) {
                match min {
                    10 => opp_gold_at_10 = Some(opf.total_gold),
                    15 => opp_gold_at_15 = Some(opf.total_gold),
                    20 => opp_gold_at_20 = Some(opf.total_gold),
                    _ => {}
                }
            }
        }
    }

    // Gold diffs vs lane opponent
    let gold_diff_at_10 = match (gold_at_10, opp_gold_at_10) {
        (Some(g), Some(o)) => Some(g - o),
        _ => None,
    };
    let gold_diff_at_15 = match (gold_at_15, opp_gold_at_15) {
        (Some(g), Some(o)) => Some(g - o),
        _ => None,
    };
    let gold_diff_at_20 = match (gold_at_20, opp_gold_at_20) {
        (Some(g), Some(o)) => Some(g - o),
        _ => None,
    };

    // Count deaths by game phase from events
    let pid_i64 = participant.participant_id as i64;
    let mut deaths_before_15 = 0i32;
    let mut deaths_after_25 = 0i32;

    for frame in &timeline.info.frames {
        for event in &frame.events {
            if event.get("type").and_then(|v| v.as_str()) == Some("CHAMPION_KILL") {
                if event.get("victimId").and_then(|v| v.as_i64()) == Some(pid_i64) {
                    let ts = event.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0);
                    let min = ts / 60000;
                    if min < 15 {
                        deaths_before_15 += 1;
                    }
                    if min >= 25 {
                        deaths_after_25 += 1;
                    }
                }
            }
        }
    }

    // Kill participation
    let team_kills: i64 = match_data
        .info
        .participants
        .iter()
        .filter(|p| p.team_id == team_id)
        .map(|p| p.kills)
        .sum();
    let kp = if team_kills > 0 {
        (participant.kills + participant.assists) as f64 / team_kills as f64
    } else {
        0.0
    };

    let vision_per_min = if game_duration_min > 0.0 {
        participant.vision_score as f64 / game_duration_min
    } else {
        0.0
    };

    let had_early_lead = gold_diff_at_15.unwrap_or(0) > 500;
    let threw_lead = had_early_lead && !participant.win;

    Some(GameFeatures {
        match_id: match_data.metadata.match_id.clone(),
        champion_id: participant.champion_id,
        champion_name: participant.champion_name.clone(),
        role: participant.team_position.clone(),
        win: participant.win,
        game_duration_min,
        cs_at_10,
        cs_at_15,
        gold_diff_at_10,
        gold_diff_at_15,
        deaths_before_15,
        gold_diff_at_20,
        deaths_after_25,
        vision_score_per_min: vision_per_min,
        kill_participation: kp,
        had_early_lead,
        threw_lead,
    })
}

/// A detected behavioral pattern
#[derive(Debug, Clone, Serialize)]
pub struct DetectedPattern {
    pub id: String,
    pub category: String,
    pub description: String,
    pub confidence: f64,
    pub sample_size: i32,
    pub impact_wr_change: Option<f64>,
    pub impact_games_pct: Option<f64>,
    pub trend: String,
}

/// Run pattern detection across all stored features for a player
pub fn detect_patterns(db: &Arc<Database>, puuid: &str) -> Vec<DetectedPattern> {
    let features = match db.get_all_features(puuid) {
        Ok(f) => f,
        Err(_) => return vec![],
    };

    if features.len() < 5 {
        return vec![]; // Need minimum data
    }

    let mut patterns = Vec::new();

    // Parse features
    let parsed: Vec<ParsedFeatures> = features
        .iter()
        .filter_map(|f| ParsedFeatures::from_json(f))
        .collect();

    if parsed.len() < 5 {
        return vec![];
    }

    // ── Pattern 1: Lead Throwing ────────────────────────
    detect_lead_throwing(&parsed, &mut patterns);

    // ── Pattern 2: Early Death Tendency ─────────────────
    detect_early_deaths(&parsed, &mut patterns);

    // ── Pattern 3: Late Game Deaths ─────────────────────
    detect_late_deaths(&parsed, &mut patterns);

    // ── Pattern 4: CS Consistency ───────────────────────
    detect_cs_patterns(&parsed, &mut patterns);

    // ── Pattern 5: Vision Control ───────────────────────
    detect_vision_patterns(&parsed, &mut patterns);

    // Store all detected patterns
    for pattern in &patterns {
        let _ = db.store_pattern(
            &pattern.id,
            puuid,
            &pattern.category,
            &pattern.description,
            pattern.confidence,
            pattern.sample_size,
            pattern.impact_wr_change,
            pattern.impact_games_pct,
            &pattern.trend,
            "[]",
        );
    }

    patterns
}

#[derive(Debug)]
struct ParsedFeatures {
    win: bool,
    game_duration_min: f64,
    gold_diff_at_15: Option<i64>,
    deaths_before_15: i32,
    deaths_after_25: i32,
    cs_at_15: Option<i64>,
    vision_per_min: f64,
    kill_participation: f64,
    had_early_lead: bool,
    threw_lead: bool,
}

impl ParsedFeatures {
    fn from_json(v: &serde_json::Value) -> Option<Self> {
        Some(Self {
            win: v.get("win")?.as_bool()?,
            game_duration_min: v.get("game_duration_min")?.as_f64()?,
            gold_diff_at_15: v.get("gold_diff_at_15").and_then(|v| v.as_i64()),
            deaths_before_15: v.get("deaths_before_15").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            deaths_after_25: v.get("deaths_after_25").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            cs_at_15: v.get("cs_at_15").and_then(|v| v.as_i64()),
            vision_per_min: v.get("vision_score_per_min").and_then(|v| v.as_f64()).unwrap_or(0.0),
            kill_participation: v.get("kill_participation").and_then(|v| v.as_f64()).unwrap_or(0.0),
            had_early_lead: v.get("had_early_lead").and_then(|v| v.as_bool()).unwrap_or(false),
            threw_lead: v.get("threw_lead").and_then(|v| v.as_bool()).unwrap_or(false),
        })
    }
}

fn detect_lead_throwing(features: &[ParsedFeatures], patterns: &mut Vec<DetectedPattern>) {
    let games_with_lead: Vec<&ParsedFeatures> = features.iter().filter(|f| f.had_early_lead).collect();
    if games_with_lead.len() < 5 {
        return;
    }
    let throws: Vec<&&ParsedFeatures> = games_with_lead.iter().filter(|f| f.threw_lead).collect();
    let throw_rate = throws.len() as f64 / games_with_lead.len() as f64;

    // If you lose more than 40% of games where you had a lead, that's a pattern
    if throw_rate > 0.40 {
        let overall_wr = features.iter().filter(|f| f.win).count() as f64 / features.len() as f64;
        let lead_wr = 1.0 - throw_rate;

        patterns.push(DetectedPattern {
            id: "lead_throwing".to_string(),
            category: "GoldManagement".to_string(),
            description: format!(
                "You lose {:.0}% of games where you have a gold lead at 15 min. \
                 Your win rate when ahead ({:.0}%) is lower than expected. \
                 You may be over-extending or not converting leads into objectives.",
                throw_rate * 100.0,
                lead_wr * 100.0,
            ),
            confidence: (games_with_lead.len() as f64 / 20.0).min(1.0),
            sample_size: games_with_lead.len() as i32,
            impact_wr_change: Some(lead_wr - overall_wr),
            impact_games_pct: Some(games_with_lead.len() as f64 / features.len() as f64),
            trend: "Stable".to_string(),
        });
    }
}

fn detect_early_deaths(features: &[ParsedFeatures], patterns: &mut Vec<DetectedPattern>) {
    let avg_early_deaths: f64 =
        features.iter().map(|f| f.deaths_before_15 as f64).sum::<f64>() / features.len() as f64;

    if avg_early_deaths > 1.5 && features.len() >= 10 {
        // Compare WR with 0-1 early deaths vs 2+ early deaths
        let low_death_games: Vec<&ParsedFeatures> = features.iter().filter(|f| f.deaths_before_15 <= 1).collect();
        let high_death_games: Vec<&ParsedFeatures> = features.iter().filter(|f| f.deaths_before_15 >= 2).collect();

        let low_wr = if !low_death_games.is_empty() {
            low_death_games.iter().filter(|f| f.win).count() as f64 / low_death_games.len() as f64
        } else { 0.5 };
        let high_wr = if !high_death_games.is_empty() {
            high_death_games.iter().filter(|f| f.win).count() as f64 / high_death_games.len() as f64
        } else { 0.5 };

        if low_wr - high_wr > 0.10 {
            patterns.push(DetectedPattern {
                id: "early_deaths".to_string(),
                category: "DeathTiming".to_string(),
                description: format!(
                    "You average {:.1} deaths before 15 minutes. \
                     Your win rate drops from {:.0}% to {:.0}% when you have 2+ early deaths. \
                     Focus on safer laning and ward coverage.",
                    avg_early_deaths,
                    low_wr * 100.0,
                    high_wr * 100.0,
                ),
                confidence: (features.len() as f64 / 20.0).min(1.0),
                sample_size: features.len() as i32,
                impact_wr_change: Some(high_wr - low_wr),
                impact_games_pct: Some(high_death_games.len() as f64 / features.len() as f64),
                trend: "Stable".to_string(),
            });
        }
    }
}

fn detect_late_deaths(features: &[ParsedFeatures], patterns: &mut Vec<DetectedPattern>) {
    let long_games: Vec<&ParsedFeatures> = features.iter().filter(|f| f.game_duration_min >= 25.0).collect();
    if long_games.len() < 5 {
        return;
    }
    let avg_late_deaths: f64 =
        long_games.iter().map(|f| f.deaths_after_25 as f64).sum::<f64>() / long_games.len() as f64;

    if avg_late_deaths > 2.0 {
        patterns.push(DetectedPattern {
            id: "late_caught_out".to_string(),
            category: "DeathTiming".to_string(),
            description: format!(
                "In games lasting 25+ minutes, you average {:.1} deaths in the late game. \
                 Getting caught out late often costs baron or the game. \
                 Stay grouped and avoid face-checking alone.",
                avg_late_deaths,
            ),
            confidence: (long_games.len() as f64 / 15.0).min(1.0),
            sample_size: long_games.len() as i32,
            impact_wr_change: None,
            impact_games_pct: Some(long_games.len() as f64 / features.len() as f64),
            trend: "Stable".to_string(),
        });
    }
}

fn detect_cs_patterns(features: &[ParsedFeatures], patterns: &mut Vec<DetectedPattern>) {
    let with_cs: Vec<&ParsedFeatures> = features.iter().filter(|f| f.cs_at_15.is_some()).collect();
    if with_cs.len() < 8 {
        return;
    }
    let avg_cs: f64 = with_cs.iter().map(|f| f.cs_at_15.unwrap() as f64).sum::<f64>() / with_cs.len() as f64;

    // Low CS at 15 (below ~100 is concerning for laners)
    if avg_cs < 95.0 {
        let good_cs: Vec<&&ParsedFeatures> = with_cs.iter().filter(|f| f.cs_at_15.unwrap() >= 100).collect();
        let bad_cs: Vec<&&ParsedFeatures> = with_cs.iter().filter(|f| f.cs_at_15.unwrap() < 90).collect();

        let good_wr = if !good_cs.is_empty() {
            good_cs.iter().filter(|f| f.win).count() as f64 / good_cs.len() as f64
        } else { 0.5 };
        let bad_wr = if !bad_cs.is_empty() {
            bad_cs.iter().filter(|f| f.win).count() as f64 / bad_cs.len() as f64
        } else { 0.5 };

        patterns.push(DetectedPattern {
            id: "low_cs".to_string(),
            category: "CsEfficiency".to_string(),
            description: format!(
                "Your average CS at 15 minutes is {:.0}. \
                 Games with 100+ CS have {:.0}% WR vs {:.0}% with <90 CS. \
                 Practice last-hitting and wave management to close this gap.",
                avg_cs,
                good_wr * 100.0,
                bad_wr * 100.0,
            ),
            confidence: (with_cs.len() as f64 / 15.0).min(1.0),
            sample_size: with_cs.len() as i32,
            impact_wr_change: Some(bad_wr - good_wr),
            impact_games_pct: Some(bad_cs.len() as f64 / with_cs.len() as f64),
            trend: "Stable".to_string(),
        });
    }
}

fn detect_vision_patterns(features: &[ParsedFeatures], patterns: &mut Vec<DetectedPattern>) {
    if features.len() < 8 {
        return;
    }
    let avg_vision: f64 =
        features.iter().map(|f| f.vision_per_min).sum::<f64>() / features.len() as f64;

    if avg_vision < 0.8 {
        let good_vision: Vec<&ParsedFeatures> = features.iter().filter(|f| f.vision_per_min >= 1.0).collect();
        let bad_vision: Vec<&ParsedFeatures> = features.iter().filter(|f| f.vision_per_min < 0.6).collect();

        let good_wr = if !good_vision.is_empty() {
            good_vision.iter().filter(|f| f.win).count() as f64 / good_vision.len() as f64
        } else { 0.5 };
        let bad_wr = if !bad_vision.is_empty() {
            bad_vision.iter().filter(|f| f.win).count() as f64 / bad_vision.len() as f64
        } else { 0.5 };

        patterns.push(DetectedPattern {
            id: "low_vision".to_string(),
            category: "VisionControl".to_string(),
            description: format!(
                "Your vision score averages {:.1}/min, which is below typical. \
                 Games with good vision (1.0+/min) have {:.0}% WR vs {:.0}% with low vision. \
                 Buy control wards and place wards proactively before fights.",
                avg_vision,
                good_wr * 100.0,
                bad_wr * 100.0,
            ),
            confidence: (features.len() as f64 / 15.0).min(1.0),
            sample_size: features.len() as i32,
            impact_wr_change: Some(bad_wr - good_wr),
            impact_games_pct: Some(bad_vision.len() as f64 / features.len() as f64),
            trend: "Stable".to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_features(win: bool, early_lead: bool, threw: bool, deaths_early: i32, cs15: i64, vision: f64) -> ParsedFeatures {
        ParsedFeatures {
            win,
            game_duration_min: 30.0,
            gold_diff_at_15: if early_lead { Some(1000) } else { Some(-500) },
            deaths_before_15: deaths_early,
            deaths_after_25: 1,
            cs_at_15: Some(cs15),
            vision_per_min: vision,
            kill_participation: 0.6,
            had_early_lead: early_lead,
            threw_lead: threw,
        }
    }

    #[test]
    fn test_detect_lead_throwing() {
        let mut patterns = Vec::new();
        // 10 games with lead, 6 thrown
        let features: Vec<ParsedFeatures> = (0..10)
            .map(|i| make_features(i >= 6, true, i < 6, 1, 110, 1.0))
            .collect();
        detect_lead_throwing(&features, &mut patterns);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].id, "lead_throwing");
    }

    #[test]
    fn test_detect_early_deaths() {
        let mut patterns = Vec::new();
        // Mix: 12 games with 3 deaths (lose), 8 games with 0 deaths (win)
        // avg deaths = (12*3)/20 = 1.8 > 1.5 threshold
        let mut features = Vec::new();
        for _ in 0..12 {
            features.push(make_features(false, false, false, 3, 110, 1.0)); // lose, high deaths
        }
        for _ in 0..8 {
            features.push(make_features(true, false, false, 0, 110, 1.0)); // win, low deaths
        }
        detect_early_deaths(&features, &mut patterns);
        assert!(patterns.iter().any(|p| p.id == "early_deaths"));
    }

    #[test]
    fn test_no_patterns_small_sample() {
        let mut patterns = Vec::new();
        let features: Vec<ParsedFeatures> = (0..3)
            .map(|_| make_features(true, true, false, 0, 110, 1.0))
            .collect();
        detect_lead_throwing(&features, &mut patterns);
        assert!(patterns.is_empty()); // Not enough data
    }
}
