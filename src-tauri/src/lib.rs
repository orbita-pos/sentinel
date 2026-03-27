mod database;
mod error;
mod lcu;

// Phase 3+
// mod riot_api;
// mod game_client;
// mod analysis;

use std::sync::Arc;

use database::Database;
use lcu::LcuManager;
use tauri::Manager;

#[tauri::command]
fn get_connection_status(lcu: tauri::State<'_, Arc<LcuManager>>) -> serde_json::Value {
    let state = lcu.get_state();
    serde_json::json!({
        "status": state.status,
        "summoner": state.summoner,
        "game_phase": state.game_phase,
    })
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn get_db_stats(db: tauri::State<'_, Database>) -> Result<serde_json::Value, String> {
    db.get_stats().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter("sentinel=debug,sentinel_lib=debug")
        .init();

    tracing::info!("Starting Sentinel v{}", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Initialize database
            let db = Database::new(&app_handle)?;
            db.run_migrations()?;
            tracing::info!("Database initialized");
            app.manage(db);

            // Initialize LCU Manager
            let lcu_manager = Arc::new(LcuManager::new(app_handle.clone()));
            app.manage(lcu_manager.clone());

            // Spawn LCU manager as long-lived background task
            tauri::async_runtime::spawn(async move {
                lcu_manager.run().await;
            });
            tracing::info!("LCU manager started");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_connection_status,
            get_app_version,
            get_db_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Sentinel");
}
