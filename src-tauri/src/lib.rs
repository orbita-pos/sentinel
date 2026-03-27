mod database;
mod error;
mod lcu;
mod riot_api;

// Phase 5+
// mod game_client;
// mod analysis;

use std::sync::Arc;

use database::Database;
use lcu::LcuManager;
use riot_api::client::RiotApiClient;
use riot_api::fetcher::MatchFetcher;
use tauri::Manager;
use tokio::sync::Mutex as AsyncMutex;

/// Shared state for the Riot API + fetcher (needs API key to initialize)
struct ApiState {
    fetcher: Option<Arc<MatchFetcher>>,
}

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
fn get_db_stats(db: tauri::State<'_, Arc<Database>>) -> Result<serde_json::Value, String> {
    db.get_stats().map_err(|e| e.to_string())
}

#[tauri::command]
fn set_api_key(
    key: String,
    db: tauri::State<'_, Arc<Database>>,
    api_state: tauri::State<'_, Arc<AsyncMutex<ApiState>>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    db.set_state("api_key", &key).map_err(|e| e.to_string())?;

    // Get region
    let platform = db
        .get_state("region")
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "la1".to_string());

    // Initialize the API client + fetcher
    let api_client = Arc::new(RiotApiClient::new(key, platform));
    let fetcher = Arc::new(MatchFetcher::new(api_client, db.inner().clone(), app_handle));

    let api_state_clone = api_state.inner().clone();
    tauri::async_runtime::block_on(async {
        let mut state = api_state_clone.lock().await;
        state.fetcher = Some(fetcher);
    });

    tracing::info!("API key configured");
    Ok(())
}

#[tauri::command]
fn set_region(
    region: String,
    db: tauri::State<'_, Arc<Database>>,
    api_state: tauri::State<'_, Arc<AsyncMutex<ApiState>>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    db.set_state("region", &region).map_err(|e| e.to_string())?;

    // Re-initialize if API key exists
    if let Ok(Some(api_key)) = db.get_state("api_key") {
        let api_client = Arc::new(RiotApiClient::new(api_key, region));
        let fetcher = Arc::new(MatchFetcher::new(api_client, db.inner().clone(), app_handle));

        let api_state_clone = api_state.inner().clone();
        tauri::async_runtime::block_on(async {
            let mut state = api_state_clone.lock().await;
            state.fetcher = Some(fetcher);
        });
    }

    Ok(())
}

#[tauri::command]
fn get_match_history(
    puuid: String,
    count: i32,
    offset: i32,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<serde_json::Value, String> {
    let matches = db.get_match_history(&puuid, count, offset).map_err(|e| e.to_string())?;
    let total = db.get_match_count(&puuid).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "matches": matches,
        "total": total,
    }))
}

#[tauri::command]
async fn trigger_backfill(
    puuid: String,
    api_state: tauri::State<'_, Arc<AsyncMutex<ApiState>>>,
) -> Result<serde_json::Value, String> {
    let fetcher = {
        let state = api_state.lock().await;
        state.fetcher.clone()
    };

    let fetcher = fetcher.ok_or("API key not configured")?;

    // Update static data first
    if let Err(e) = fetcher.update_static_data().await {
        tracing::warn!("Failed to update static data: {e}");
    }

    // Run backfill
    let count = fetcher
        .backfill(&puuid, 20)
        .await
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({ "fetched": count }))
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
            let db = Arc::new(Database::new(&app_handle)?);
            db.run_migrations()?;
            tracing::info!("Database initialized");
            app.manage(db.clone());

            // Initialize API state (no fetcher until API key is set)
            let api_state = Arc::new(AsyncMutex::new(ApiState { fetcher: None }));

            // Try to restore API key from database
            if let Ok(Some(api_key)) = db.get_state("api_key") {
                let platform = db
                    .get_state("region")
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| "la1".to_string());
                let api_client = Arc::new(RiotApiClient::new(api_key, platform));
                let fetcher = Arc::new(MatchFetcher::new(
                    api_client,
                    db.clone(),
                    app_handle.clone(),
                ));
                tauri::async_runtime::block_on(async {
                    let mut state = api_state.lock().await;
                    state.fetcher = Some(fetcher);
                });
                tracing::info!("Restored API key from database");
            }

            app.manage(api_state);

            // Initialize LCU Manager
            let lcu_manager = Arc::new(LcuManager::new(app_handle.clone()));
            app.manage(lcu_manager.clone());

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
            set_api_key,
            set_region,
            get_match_history,
            trigger_backfill,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Sentinel");
}
