use std::sync::Arc;

use serde::Serialize;

use crate::database::{ChampionPoolEntry, Database};
use crate::lcu::champ_select::ChampSelectSession;

/// A draft recommendation for the local player
#[derive(Debug, Clone, Serialize)]
pub struct DraftRecommendation {
    pub champion_id: i64,
    pub champion_name: String,
    pub score: f64,         // 0-100 composite score
    pub personal_wr: f64,   // 0-1
    pub personal_games: i64,
    pub reasons: Vec<String>,
}

/// Analyze the current draft and produce champion recommendations
pub fn get_recommendations(
    session: &ChampSelectSession,
    db: &Arc<Database>,
    puuid: &str,
) -> Vec<DraftRecommendation> {
    // Get personal champion pool (minimum 2 games for a recommendation)
    let pool = match db.get_champion_pool(puuid, 2) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    if pool.is_empty() {
        return vec![];
    }

    // Gather context about the current draft
    let banned_ids: Vec<i64> = session.bans.clone();
    let picked_ids: Vec<i64> = session
        .my_team
        .iter()
        .chain(session.their_team.iter())
        .map(|p| p.champion_id)
        .filter(|id| *id > 0)
        .collect();

    // Find local player's assigned role
    let my_role = session
        .my_team
        .iter()
        .find(|p| p.is_local_player)
        .map(|p| p.assigned_position.clone())
        .unwrap_or_default();

    // Score each champion in the pool
    let mut recommendations: Vec<DraftRecommendation> = pool
        .iter()
        .filter(|c| {
            // Exclude banned and already picked champions
            !banned_ids.contains(&c.champion_id) && !picked_ids.contains(&c.champion_id)
        })
        .map(|c| score_champion(c, &my_role, session))
        .collect();

    // Sort by score descending
    recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    // Return top 5
    recommendations.truncate(5);
    recommendations
}

/// Score a single champion for the current draft context
fn score_champion(
    champ: &ChampionPoolEntry,
    my_role: &str,
    _session: &ChampSelectSession,
) -> DraftRecommendation {
    let mut score = 0.0;
    let mut reasons = Vec::new();

    // ── Win rate component (0-40 points) ───────────────────
    let wr_score = champ.win_rate * 40.0;
    score += wr_score;
    if champ.win_rate >= 0.55 {
        reasons.push(format!("{:.0}% win rate ({} games)", champ.win_rate * 100.0, champ.games));
    }

    // ── Games played / comfort (0-30 points) ───────────────
    // Logarithmic scaling: more games = higher comfort, but diminishing returns
    let comfort = (champ.games as f64).ln().min(4.0) / 4.0 * 30.0;
    score += comfort;
    if champ.games >= 10 {
        reasons.push(format!("{} games played (high comfort)", champ.games));
    } else if champ.games >= 5 {
        reasons.push(format!("{} games played", champ.games));
    }

    // ── KDA component (0-15 points) ────────────────────────
    let kda = if champ.avg_deaths > 0.0 {
        (champ.avg_kills + champ.avg_assists) / champ.avg_deaths
    } else {
        (champ.avg_kills + champ.avg_assists) * 2.0
    };
    let kda_score = (kda / 5.0).min(1.0) * 15.0;
    score += kda_score;
    if kda >= 3.0 {
        reasons.push(format!("{:.1} KDA average", kda));
    }

    // ── CS component (0-10 points, mainly for laners) ──────
    if !my_role.is_empty() && my_role != "JUNGLE" && my_role != "UTILITY" {
        let cs_score = (champ.avg_cs / 7.0).min(1.0) * 10.0; // 7 cs/game-minute ~= good
        score += cs_score;
        if champ.avg_cs >= 6.0 {
            reasons.push(format!("{:.1} avg CS/min", champ.avg_cs));
        }
    }

    // ── Recency bonus (0-5 points) ─────────────────────────
    // More games = likely played recently (simple heuristic without timestamps)
    if champ.games >= 5 {
        score += 5.0;
    }

    // Clamp to 0-100
    score = score.clamp(0.0, 100.0);

    if reasons.is_empty() {
        reasons.push("In your champion pool".to_string());
    }

    DraftRecommendation {
        champion_id: champ.champion_id,
        champion_name: champ.champion_name.clone(),
        score,
        personal_wr: champ.win_rate,
        personal_games: champ.games,
        reasons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_champion_high_wr() {
        let champ = ChampionPoolEntry {
            champion_id: 39,
            champion_name: "Irelia".to_string(),
            games: 50,
            wins: 35,
            win_rate: 0.70,
            avg_kills: 8.0,
            avg_deaths: 4.0,
            avg_assists: 6.0,
            avg_cs: 7.5,
        };
        let session = ChampSelectSession::default();
        let rec = score_champion(&champ, "TOP", &session);

        assert!(rec.score > 60.0); // High WR + high games should score well
        assert!(rec.reasons.iter().any(|r| r.contains("win rate")));
        assert!(rec.reasons.iter().any(|r| r.contains("games played")));
    }

    #[test]
    fn test_score_champion_low_games() {
        let champ = ChampionPoolEntry {
            champion_id: 86,
            champion_name: "Garen".to_string(),
            games: 2,
            wins: 2,
            win_rate: 1.0,
            avg_kills: 5.0,
            avg_deaths: 3.0,
            avg_assists: 4.0,
            avg_cs: 5.0,
        };
        let session = ChampSelectSession::default();
        let rec = score_champion(&champ, "TOP", &session);

        // 100% WR but only 2 games -- score should reflect low confidence
        assert!(rec.score < 80.0);
    }
}
