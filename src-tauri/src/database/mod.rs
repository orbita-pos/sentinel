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
}
