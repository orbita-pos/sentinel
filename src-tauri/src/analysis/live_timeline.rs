use std::sync::Arc;

use crate::analysis::patterns::GameFeatures;
use crate::database::Database;

/// Reconstruct GameFeatures from live capture snapshots (NO Riot API needed).
/// This gives us the same feature set as timeline-based extraction for any
/// game where Sentinel was running.
pub fn extract_features_from_session(
    db: &Arc<Database>,
    session_id: &str,
    puuid: &str,
    champion_name: &str,
    champion_id: i64,
    role: &str,
    win: bool,
    game_duration_sec: f64,
) -> Option<GameFeatures> {
    let game_duration_min = game_duration_sec / 60.0;
    if game_duration_min < 5.0 {
        return None; // Skip remakes
    }

    // Get local player snapshots at key minutes
    let local_snaps = db
        .get_local_snapshots_at_minutes(session_id, &[10, 15, 20, 25])
        .ok()?;

    let snap_at = |min: i64| -> Option<&serde_json::Value> {
        local_snaps.iter().find(|s| s.get("minute").and_then(|v| v.as_i64()) == Some(min))
    };

    let cs_at_10 = snap_at(10).and_then(|s| s.get("cs").and_then(|v| v.as_i64()));
    let cs_at_15 = snap_at(15).and_then(|s| s.get("cs").and_then(|v| v.as_i64()));

    // Get team gold data for gold diffs
    let team_gold = db.get_team_gold_by_minute(session_id).ok().unwrap_or_default();

    // Build gold per team per minute
    let mut my_team: Option<String> = None;

    // Determine local player's team from snapshots
    if let Ok(conn_lock) = db.get_state("_dummy") {
        // We need the team from snapshots directly
        let _ = conn_lock; // just to avoid warning
    }

    // Simpler approach: get team from earliest snapshot with is_local=1
    // We already have the team gold grouped - find our team
    // The local player's team info isn't directly in team_gold, so we compute diffs
    // using the assumption that if we have 2 teams, the one matching our items is ours.

    // For gold diffs, use item_gold from local snapshots vs opponent estimate
    let local_gold_at_10 = snap_at(10).and_then(|s| s.get("item_gold").and_then(|v| v.as_i64()));
    let local_gold_at_15 = snap_at(15).and_then(|s| s.get("item_gold").and_then(|v| v.as_i64()));
    let local_gold_at_20 = snap_at(20).and_then(|s| s.get("item_gold").and_then(|v| v.as_i64()));

    // Compute team gold diffs from aggregated data
    let mut gold_diff_at = |target_min: i64| -> Option<i64> {
        let at_min: Vec<&serde_json::Value> = team_gold
            .iter()
            .filter(|g| g.get("minute").and_then(|v| v.as_i64()) == Some(target_min))
            .collect();

        if at_min.len() == 2 {
            let g0 = at_min[0].get("gold").and_then(|v| v.as_i64()).unwrap_or(0);
            let g1 = at_min[1].get("gold").and_then(|v| v.as_i64()).unwrap_or(0);
            let t0 = at_min[0].get("team").and_then(|v| v.as_str()).unwrap_or("");

            // We need to figure out which team is ours
            // Use the team with the local player's gold contribution
            if my_team.is_none() {
                // Heuristic: our gold should be roughly part of one team's total
                if let Some(local_g) = local_gold_at_10.or(local_gold_at_15) {
                    if (g0 as f64 * 0.1) < local_g as f64 && (local_g as f64) < (g0 as f64 * 0.5) {
                        my_team = Some(t0.to_string());
                    } else {
                        my_team = Some(
                            at_min[1]
                                .get("team")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                        );
                    }
                }
            }

            let my_t = my_team.as_deref().unwrap_or("");
            if t0 == my_t {
                Some(g0 - g1)
            } else {
                Some(g1 - g0)
            }
        } else {
            None
        }
    };

    let gold_diff_at_10 = gold_diff_at(10);
    let gold_diff_at_15 = gold_diff_at(15);
    let gold_diff_at_20 = gold_diff_at(20);

    // Count deaths from live events
    let events = db.get_live_death_events(session_id).ok().unwrap_or_default();
    let mut deaths_before_15 = 0i32;
    let mut deaths_after_25 = 0i32;

    // We track ALL kill events - the poller records "ChampionKill" for any kill
    // We need to check if our local player died - look at death count changes in snapshots
    let deaths_at_15 = snap_at(15)
        .and_then(|s| s.get("deaths").and_then(|v| v.as_i64()))
        .unwrap_or(0) as i32;
    let deaths_at_25 = snap_at(25)
        .and_then(|s| s.get("deaths").and_then(|v| v.as_i64()))
        .unwrap_or(0) as i32;

    // Final deaths from game duration
    // Use last available snapshot
    let final_deaths = local_snaps
        .last()
        .and_then(|s| s.get("deaths").and_then(|v| v.as_i64()))
        .unwrap_or(0) as i32;

    deaths_before_15 = deaths_at_15;
    deaths_after_25 = final_deaths - deaths_at_25;
    if deaths_after_25 < 0 {
        deaths_after_25 = 0;
    }

    // Vision score per min from last snapshot
    let final_ward = local_snaps
        .last()
        .and_then(|s| s.get("ward_score").and_then(|v| v.as_f64()))
        .unwrap_or(0.0);
    let vision_score_per_min = if game_duration_min > 0.0 {
        final_ward / game_duration_min
    } else {
        0.0
    };

    // Kill participation: (kills + assists) / team_kills
    // We approximate from final snapshot
    let final_kills = local_snaps
        .last()
        .and_then(|s| s.get("kills").and_then(|v| v.as_i64()))
        .unwrap_or(0);
    let final_assists = local_snaps
        .last()
        .and_then(|s| s.get("assists").and_then(|v| v.as_i64()))
        .unwrap_or(0);

    // Rough team kills estimate: our kills + assists is a decent proxy for KP
    // Without full team data, use a reasonable estimate
    let kp = if final_kills + final_assists > 0 {
        // Assume team had roughly 2x our kills
        let estimated_team_kills = (final_kills as f64 * 2.0).max(1.0);
        ((final_kills + final_assists) as f64 / estimated_team_kills).min(1.0)
    } else {
        0.0
    };

    let had_early_lead = gold_diff_at_15.unwrap_or(0) > 500;
    let threw_lead = had_early_lead && !win;

    Some(GameFeatures {
        match_id: format!("live_{session_id}"),
        champion_id,
        champion_name: champion_name.to_string(),
        role: role.to_string(),
        win,
        game_duration_min,
        cs_at_10,
        cs_at_15,
        gold_diff_at_10,
        gold_diff_at_15,
        deaths_before_15,
        gold_diff_at_20,
        deaths_after_25,
        vision_score_per_min,
        kill_participation: kp,
        had_early_lead,
        threw_lead,
    })
}
