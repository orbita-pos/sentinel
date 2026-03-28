use std::sync::Arc;

use serde::Serialize;

use crate::database::Database;

/// Weekly improvement metrics
#[derive(Debug, Clone, Serialize)]
pub struct WeeklyMetrics {
    pub metrics: Vec<MetricSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricSummary {
    pub key: String,
    pub label: String,
    pub current_value: f64,
    pub previous_value: Option<f64>,
    pub trend_pct: Option<f64>,
    pub unit: String,
}

/// Compute current metrics from game features for a player
pub fn compute_metrics(db: &Arc<Database>, puuid: &str) -> WeeklyMetrics {
    let features = db.get_all_features(puuid).unwrap_or_default();

    if features.is_empty() {
        return WeeklyMetrics { metrics: vec![] };
    }

    let total = features.len() as f64;

    // Win rate
    let wins = features.iter().filter(|f| f.get("win").and_then(|v| v.as_bool()).unwrap_or(false)).count();
    let wr = wins as f64 / total;

    // Average CS at 15
    let cs_vals: Vec<f64> = features.iter()
        .filter_map(|f| f.get("cs_at_15").and_then(|v| v.as_f64()))
        .collect();
    let avg_cs_15 = if !cs_vals.is_empty() { cs_vals.iter().sum::<f64>() / cs_vals.len() as f64 } else { 0.0 };

    // Average deaths before 15
    let deaths_vals: Vec<f64> = features.iter()
        .filter_map(|f| f.get("deaths_before_15").and_then(|v| v.as_f64()))
        .collect();
    let avg_early_deaths = if !deaths_vals.is_empty() { deaths_vals.iter().sum::<f64>() / deaths_vals.len() as f64 } else { 0.0 };

    // Average vision score per minute
    let vision_vals: Vec<f64> = features.iter()
        .filter_map(|f| f.get("vision_score_per_min").and_then(|v| v.as_f64()))
        .collect();
    let avg_vision = if !vision_vals.is_empty() { vision_vals.iter().sum::<f64>() / vision_vals.len() as f64 } else { 0.0 };

    // Average kill participation
    let kp_vals: Vec<f64> = features.iter()
        .filter_map(|f| f.get("kill_participation").and_then(|v| v.as_f64()))
        .collect();
    let avg_kp = if !kp_vals.is_empty() { kp_vals.iter().sum::<f64>() / kp_vals.len() as f64 } else { 0.0 };

    // Lead conversion rate
    let leads: Vec<bool> = features.iter()
        .filter(|f| f.get("had_early_lead").and_then(|v| v.as_bool()).unwrap_or(false))
        .map(|f| f.get("win").and_then(|v| v.as_bool()).unwrap_or(false))
        .collect();
    let lead_conversion = if !leads.is_empty() {
        leads.iter().filter(|&&w| w).count() as f64 / leads.len() as f64
    } else { 0.0 };

    // Store today's snapshot
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let n = features.len() as i32;
    let _ = db.store_improvement_snapshot(puuid, "win_rate", wr, n, &today);
    let _ = db.store_improvement_snapshot(puuid, "cs_at_15", avg_cs_15, n, &today);
    let _ = db.store_improvement_snapshot(puuid, "early_deaths", avg_early_deaths, n, &today);
    let _ = db.store_improvement_snapshot(puuid, "vision_per_min", avg_vision, n, &today);
    let _ = db.store_improvement_snapshot(puuid, "kill_participation", avg_kp, n, &today);
    let _ = db.store_improvement_snapshot(puuid, "lead_conversion", lead_conversion, n, &today);

    // Get previous snapshots for trend calculation
    let metrics = vec![
        build_metric(db, puuid, "win_rate", "Win Rate", wr, "%", 100.0),
        build_metric(db, puuid, "cs_at_15", "CS at 15 min", avg_cs_15, "cs", 1.0),
        build_metric(db, puuid, "early_deaths", "Deaths before 15", avg_early_deaths, "avg", 1.0),
        build_metric(db, puuid, "vision_per_min", "Vision Score/min", avg_vision, "/min", 1.0),
        build_metric(db, puuid, "kill_participation", "Kill Participation", avg_kp, "%", 100.0),
        build_metric(db, puuid, "lead_conversion", "Lead Conversion", lead_conversion, "%", 100.0),
    ];

    WeeklyMetrics { metrics }
}

fn build_metric(
    db: &Arc<Database>,
    puuid: &str,
    key: &str,
    label: &str,
    current: f64,
    unit: &str,
    display_mult: f64,
) -> MetricSummary {
    // Get previous snapshot (skip today's, get the one before)
    let snapshots = db.get_improvement_snapshots(puuid, key, 7).unwrap_or_default();
    let previous = snapshots.iter()
        .skip(1) // skip today
        .next()
        .and_then(|s| s.get("value").and_then(|v| v.as_f64()));

    let trend_pct = previous.map(|prev| {
        if prev.abs() < 0.001 { 0.0 } else { (current - prev) / prev * 100.0 }
    });

    MetricSummary {
        key: key.to_string(),
        label: label.to_string(),
        current_value: current * display_mult,
        previous_value: previous.map(|p| p * display_mult),
        trend_pct,
        unit: unit.to_string(),
    }
}
