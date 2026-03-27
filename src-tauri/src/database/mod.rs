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
