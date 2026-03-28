mod migrations;
mod schema;

use crate::error::AppError;
use rusqlite::Connection;
use tauri::Manager;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
    path: PathBuf,
}

impl Database {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {e}"))?;

        std::fs::create_dir_all(&app_dir)?;

        let db_path = app_dir.join("sentinel.db");
        tracing::info!("Database path: {}", db_path.display());

        let conn = Connection::open(&db_path)?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        Ok(Self {
            conn: Mutex::new(conn),
            path: db_path,
        })
    }

    pub fn run_migrations(&self) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        migrations::run_all(&conn)
    }

    pub fn get_stats(&self) -> Result<serde_json::Value, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;

        let summoner_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM summoners", [], |row| row.get(0))
            .unwrap_or(0);

        let state_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM app_state", [], |row| row.get(0))
            .unwrap_or(0);

        Ok(serde_json::json!({
            "db_path": self.path.to_string_lossy(),
            "summoners": summoner_count,
            "app_state_entries": state_count,
        }))
    }

    /// Get a value from app_state
    pub fn get_state(&self, key: &str) -> Result<Option<String>, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT value FROM app_state WHERE key = ?1")?;
        let result = stmt.query_row([key], |row| row.get::<_, String>(0));
        match result {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Set a value in app_state
    pub fn set_state(&self, key: &str, value: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        conn.execute(
            "INSERT INTO app_state (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [key, value],
        )?;
        Ok(())
    }

    /// Check if a match exists in the database
    pub fn has_match(&self, match_id: &str) -> Result<bool, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM matches WHERE match_id = ?1",
            [match_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Store a complete match with participants
    pub fn store_match(
        &self,
        match_id: &str,
        game_creation: i64,
        game_duration: i64,
        game_mode: &str,
        queue_id: i64,
        game_version: Option<&str>,
        raw_json: &str,
        participants: &[MatchParticipantRow],
    ) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        conn.execute(
            "INSERT OR IGNORE INTO matches (match_id, game_creation, game_duration, game_mode, queue_id, game_version, raw_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![match_id, game_creation, game_duration, game_mode, queue_id, game_version, raw_json],
        )?;
        for p in participants {
            conn.execute(
                "INSERT OR IGNORE INTO match_participants
                 (match_id, participant_id, puuid, champion_id, champion_name, team_id, team_position,
                  kills, deaths, assists, total_minions_killed, gold_earned, total_damage_dealt, vision_score, win)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                rusqlite::params![
                    match_id, p.participant_id, p.puuid, p.champion_id, p.champion_name,
                    p.team_id, p.team_position, p.kills, p.deaths, p.assists,
                    p.total_minions_killed, p.gold_earned, p.total_damage_dealt, p.vision_score,
                    p.win as i32
                ],
            )?;
        }
        Ok(())
    }

    /// Store a match timeline
    pub fn store_timeline(&self, match_id: &str, raw_json: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        conn.execute(
            "INSERT OR IGNORE INTO match_timelines (match_id, raw_json) VALUES (?1, ?2)",
            [match_id, raw_json],
        )?;
        Ok(())
    }

    /// Get match history for a player (most recent first)
    pub fn get_match_history(&self, puuid: &str, count: i32, offset: i32) -> Result<Vec<MatchSummary>, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT m.match_id, m.game_creation, m.game_duration, m.game_mode, m.queue_id,
                    mp.champion_name, mp.champion_id, mp.kills, mp.deaths, mp.assists,
                    mp.total_minions_killed, mp.gold_earned, mp.vision_score, mp.win, mp.team_position
             FROM matches m
             JOIN match_participants mp ON m.match_id = mp.match_id AND mp.puuid = ?1
             ORDER BY m.game_creation DESC
             LIMIT ?2 OFFSET ?3"
        )?;
        let rows = stmt.query_map(rusqlite::params![puuid, count, offset], |row| {
            Ok(MatchSummary {
                match_id: row.get(0)?,
                game_creation: row.get(1)?,
                game_duration: row.get(2)?,
                game_mode: row.get(3)?,
                queue_id: row.get(4)?,
                champion_name: row.get(5)?,
                champion_id: row.get(6)?,
                kills: row.get(7)?,
                deaths: row.get(8)?,
                assists: row.get(9)?,
                cs: row.get(10)?,
                gold: row.get(11)?,
                vision_score: row.get(12)?,
                win: row.get::<_, i32>(13)? != 0,
                role: row.get(14)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }

    /// Get match count for a player
    pub fn get_match_count(&self, puuid: &str) -> Result<i64, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM match_participants WHERE puuid = ?1",
            [puuid],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Store champion data (upsert)
    pub fn store_champions(&self, champions: &[(i64, &str, &str, Option<&str>, Option<&str>, &str)]) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        for (id, key, name, title, tags, patch) in champions {
            conn.execute(
                "INSERT OR REPLACE INTO champions (champion_id, champion_key, name, title, tags, patch_version)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![id, key, name, title, tags, patch],
            )?;
        }
        Ok(())
    }

    /// Store item data (upsert)
    pub fn store_items(&self, items: &[(i64, &str, Option<&str>, Option<i64>, Option<i64>, Option<&str>, Option<&str>, Option<&str>, &str)]) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        for (id, name, desc, gold_total, gold_base, tags, from_items, into_items, patch) in items {
            conn.execute(
                "INSERT OR REPLACE INTO items (item_id, name, description, gold_total, gold_base, tags, from_items, into_items, patch_version)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![id, name, desc, gold_total, gold_base, tags, from_items, into_items, patch],
            )?;
        }
        Ok(())
    }

    /// Get personal champion pool stats for a player (win rate, games played per champion)
    pub fn get_champion_pool(&self, puuid: &str, min_games: i32) -> Result<Vec<ChampionPoolEntry>, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT mp.champion_id, mp.champion_name,
                    COUNT(*) as games,
                    SUM(mp.win) as wins,
                    AVG(mp.kills) as avg_kills,
                    AVG(mp.deaths) as avg_deaths,
                    AVG(mp.assists) as avg_assists,
                    AVG(mp.total_minions_killed) as avg_cs
             FROM match_participants mp
             JOIN matches m ON m.match_id = mp.match_id
             WHERE mp.puuid = ?1
             GROUP BY mp.champion_id, mp.champion_name
             HAVING games >= ?2
             ORDER BY games DESC"
        )?;
        let rows = stmt.query_map(rusqlite::params![puuid, min_games], |row| {
            let games: i64 = row.get(2)?;
            let wins: i64 = row.get(3)?;
            Ok(ChampionPoolEntry {
                champion_id: row.get(0)?,
                champion_name: row.get(1)?,
                games,
                wins,
                win_rate: if games > 0 { wins as f64 / games as f64 } else { 0.0 },
                avg_kills: row.get(4)?,
                avg_deaths: row.get(5)?,
                avg_assists: row.get(6)?,
                avg_cs: row.get(7)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }

    /// Get champion name by ID from static data
    pub fn get_champion_name(&self, champion_id: i64) -> Result<Option<String>, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let result = conn.query_row(
            "SELECT name FROM champions WHERE champion_id = ?1",
            [champion_id],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(name) => Ok(Some(name)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    // ── Pattern Engine DB Methods ─────────────────────────

    /// Check if features are already extracted for a match
    pub fn has_features(&self, match_id: &str, puuid: &str) -> Result<bool, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM game_features WHERE match_id = ?1 AND puuid = ?2",
            rusqlite::params![match_id, puuid],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Store extracted game features
    pub fn store_features(&self, match_id: &str, puuid: &str, features_json: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let f: serde_json::Value = serde_json::from_str(features_json)?;
        conn.execute(
            "INSERT OR REPLACE INTO game_features
             (match_id, puuid, champion_id, role, win, game_duration_min,
              cs_at_10, cs_at_15, gold_diff_at_10, gold_diff_at_15, gold_diff_at_20,
              deaths_before_15, deaths_after_25, vision_score_per_min, kill_participation,
              had_early_lead, threw_lead, features_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            rusqlite::params![
                match_id, puuid,
                f.get("champion_id").and_then(|v| v.as_i64()).unwrap_or(0),
                f.get("role").and_then(|v| v.as_str()).unwrap_or(""),
                f.get("win").and_then(|v| v.as_bool()).unwrap_or(false) as i32,
                f.get("game_duration_min").and_then(|v| v.as_f64()).unwrap_or(0.0),
                f.get("cs_at_10").and_then(|v| v.as_i64()),
                f.get("cs_at_15").and_then(|v| v.as_i64()),
                f.get("gold_diff_at_10").and_then(|v| v.as_i64()),
                f.get("gold_diff_at_15").and_then(|v| v.as_i64()),
                f.get("gold_diff_at_20").and_then(|v| v.as_i64()),
                f.get("deaths_before_15").and_then(|v| v.as_i64()),
                f.get("deaths_after_25").and_then(|v| v.as_i64()),
                f.get("vision_score_per_min").and_then(|v| v.as_f64()),
                f.get("kill_participation").and_then(|v| v.as_f64()),
                f.get("had_early_lead").and_then(|v| v.as_bool()).unwrap_or(false) as i32,
                f.get("threw_lead").and_then(|v| v.as_bool()).unwrap_or(false) as i32,
                features_json,
            ],
        )?;
        Ok(())
    }

    /// Get all game features for a player
    pub fn get_all_features(&self, puuid: &str) -> Result<Vec<serde_json::Value>, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT features_json FROM game_features WHERE puuid = ?1")?;
        let rows = stmt.query_map([puuid], |row| {
            let json_str: String = row.get(0)?;
            Ok(json_str)
        })?;
        let mut results = Vec::new();
        for row in rows {
            if let Ok(json_str) = row {
                if let Ok(val) = serde_json::from_str(&json_str) {
                    results.push(val);
                }
            }
        }
        Ok(results)
    }

    /// Store or update a detected pattern
    pub fn store_pattern(&self, id: &str, puuid: &str, category: &str, description: &str,
                          confidence: f64, sample_size: i32, impact_wr: Option<f64>,
                          impact_pct: Option<f64>, trend: &str, evidence_json: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        conn.execute(
            "INSERT INTO detected_patterns (id, puuid, category, description, confidence, sample_size,
             impact_wr_change, impact_games_pct, trend, evidence_json, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, datetime('now'))
             ON CONFLICT(id) DO UPDATE SET
               description=excluded.description, confidence=excluded.confidence,
               sample_size=excluded.sample_size, impact_wr_change=excluded.impact_wr_change,
               impact_games_pct=excluded.impact_games_pct, trend=excluded.trend,
               evidence_json=excluded.evidence_json, last_updated=datetime('now')",
            rusqlite::params![id, puuid, category, description, confidence, sample_size,
                              impact_wr, impact_pct, trend, evidence_json],
        )?;
        Ok(())
    }

    /// Get all detected patterns for a player
    pub fn get_patterns(&self, puuid: &str) -> Result<Vec<serde_json::Value>, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, category, description, confidence, sample_size,
                    impact_wr_change, impact_games_pct, trend, evidence_json, first_detected, last_updated
             FROM detected_patterns WHERE puuid = ?1 ORDER BY confidence DESC"
        )?;
        let rows = stmt.query_map([puuid], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "category": row.get::<_, String>(1)?,
                "description": row.get::<_, String>(2)?,
                "confidence": row.get::<_, f64>(3)?,
                "sample_size": row.get::<_, i32>(4)?,
                "impact_wr_change": row.get::<_, Option<f64>>(5)?,
                "impact_games_pct": row.get::<_, Option<f64>>(6)?,
                "trend": row.get::<_, String>(7)?,
                "evidence_json": row.get::<_, String>(8)?,
                "first_detected": row.get::<_, String>(9)?,
                "last_updated": row.get::<_, String>(10)?,
            }))
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }

    /// Store a post-game analysis
    pub fn store_post_game_analysis(&self, match_id: &str, puuid: &str, analysis_json: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO post_game_analyses (match_id, puuid, analysis_json) VALUES (?1, ?2, ?3)",
            rusqlite::params![match_id, puuid, analysis_json],
        )?;
        Ok(())
    }

    /// Get a post-game analysis
    pub fn get_post_game_analysis(&self, match_id: &str) -> Result<Option<String>, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let result = conn.query_row(
            "SELECT analysis_json FROM post_game_analyses WHERE match_id = ?1",
            [match_id],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(json) => Ok(Some(json)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get timeline raw JSON for a match
    pub fn get_timeline_json(&self, match_id: &str) -> Result<Option<String>, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let result = conn.query_row(
            "SELECT raw_json FROM match_timelines WHERE match_id = ?1",
            [match_id],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(json) => Ok(Some(json)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get raw match JSON
    pub fn get_match_json(&self, match_id: &str) -> Result<Option<String>, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let result = conn.query_row(
            "SELECT raw_json FROM matches WHERE match_id = ?1",
            [match_id],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(json) => Ok(Some(json)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Store an improvement snapshot
    pub fn store_improvement_snapshot(&self, puuid: &str, metric_key: &str, value: f64, match_count: i32, date: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        conn.execute(
            "INSERT INTO improvement_snapshots (puuid, metric_key, value, match_count, snapshot_date)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(puuid, metric_key, snapshot_date) DO UPDATE SET value=excluded.value, match_count=excluded.match_count",
            rusqlite::params![puuid, metric_key, value, match_count, date],
        )?;
        Ok(())
    }

    /// Get improvement snapshots for a metric
    pub fn get_improvement_snapshots(&self, puuid: &str, metric_key: &str, limit: i32) -> Result<Vec<serde_json::Value>, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT snapshot_date, value, match_count FROM improvement_snapshots
             WHERE puuid = ?1 AND metric_key = ?2 ORDER BY snapshot_date DESC LIMIT ?3"
        )?;
        let rows = stmt.query_map(rusqlite::params![puuid, metric_key, limit], |row| {
            Ok(serde_json::json!({
                "date": row.get::<_, String>(0)?,
                "value": row.get::<_, f64>(1)?,
                "match_count": row.get::<_, i32>(2)?,
            }))
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }

    /// Store an improvement goal
    pub fn create_goal(&self, puuid: &str, name: &str, description: Option<&str>,
                        metric_key: &str, target_value: Option<f64>, linked_pattern: Option<&str>) -> Result<i64, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        conn.execute(
            "INSERT INTO improvement_goals (puuid, name, description, metric_key, target_value, linked_pattern_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![puuid, name, description, metric_key, target_value, linked_pattern],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Get active improvement goals
    pub fn get_goals(&self, puuid: &str) -> Result<Vec<serde_json::Value>, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, metric_key, target_value, linked_pattern_id, created_at
             FROM improvement_goals WHERE puuid = ?1 AND active = 1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map([puuid], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "name": row.get::<_, String>(1)?,
                "description": row.get::<_, Option<String>>(2)?,
                "metric_key": row.get::<_, String>(3)?,
                "target_value": row.get::<_, Option<f64>>(4)?,
                "linked_pattern_id": row.get::<_, Option<String>>(5)?,
                "created_at": row.get::<_, String>(6)?,
            }))
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }
}

/// Row data for inserting a match participant
pub struct MatchParticipantRow {
    pub participant_id: i32,
    pub puuid: String,
    pub champion_id: i64,
    pub champion_name: String,
    pub team_id: i64,
    pub team_position: String,
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
    pub total_minions_killed: i64,
    pub gold_earned: i64,
    pub total_damage_dealt: i64,
    pub vision_score: i64,
    pub win: bool,
}

/// Summary of a match for display in match history
#[derive(Debug, Clone, serde::Serialize)]
pub struct MatchSummary {
    pub match_id: String,
    pub game_creation: i64,
    pub game_duration: i64,
    pub game_mode: String,
    pub queue_id: i64,
    pub champion_name: String,
    pub champion_id: i64,
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
    pub cs: i64,
    pub gold: i64,
    pub vision_score: i64,
    pub win: bool,
    pub role: Option<String>,
}

/// Champion pool entry with aggregated stats
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChampionPoolEntry {
    pub champion_id: i64,
    pub champion_name: String,
    pub games: i64,
    pub wins: i64,
    pub win_rate: f64,
    pub avg_kills: f64,
    pub avg_deaths: f64,
    pub avg_assists: f64,
    pub avg_cs: f64,
}
