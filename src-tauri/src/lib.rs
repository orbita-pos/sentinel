mod database;
mod error;

// Phase 2+
// mod lcu;
// mod riot_api;
// mod game_client;
// mod analysis;

use database::Database;
use tauri::Manager;

#[tauri::command]
fn get_connection_status() -> String {
    "disconnected".to_string()
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
