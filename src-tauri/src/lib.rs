mod analysis;
mod database;
mod error;
mod game_client;
mod lcu;
mod riot_api;

use std::sync::Arc;

use database::Database;
use lcu::LcuManager;
use riot_api::client::RiotApiClient;
use riot_api::fetcher::MatchFetcher;
use tauri::{Emitter, Manager};
use tokio::sync::Mutex as AsyncMutex;

// ── Security helpers ──────────────────────────────────────

/// [H2-H3] Allowlist of valid Riot platform regions
const VALID_REGIONS: &[&str] = &[
    "na1", "euw1", "eun1", "kr", "br1", "la1", "la2",
    "oc1", "tr1", "ru", "jp1", "ph2", "sg2", "th2", "tw2", "vn2",
];

/// [M1] Log the real error, return a safe message to the frontend
fn safe_err(context: &str, e: impl std::fmt::Display) -> String {
    tracing::error!("{context}: {e}");
    format!("{context}: operation failed")
}

/// [M4] Validate puuid format (alphanumeric + hyphens, reasonable length)
fn validate_puuid(puuid: &str) -> Result<(), String> {
    if puuid.is_empty() || puuid.len() > 100 {
        return Err("Invalid player ID".to_string());
    }
    if !puuid.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err("Invalid player ID format".to_string());
    }
    Ok(())
}

/// [H2] Validate region against allowlist
fn validate_region(region: &str) -> Result<(), String> {
    if !VALID_REGIONS.contains(&region) {
        return Err(format!("Invalid region: {region}"));
    }
    Ok(())
}

// ── State ─────────────────────────────────────────────────

struct ApiState {
    fetcher: Option<Arc<MatchFetcher>>,
}

// ── Tauri Commands ────────────────────────────────────────

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
    db.get_stats().map_err(|e| safe_err("Database stats", e))
}

#[tauri::command]
fn get_champion_map(db: tauri::State<'_, Arc<Database>>) -> Result<serde_json::Value, String> {
    let map = db.get_champion_map().map_err(|e| safe_err("Champion map", e))?;
    serde_json::to_value(&map).map_err(|e| safe_err("Serialize champion map", e))
}

/// [H4] Now async (was sync with block_on). [C1] Uses encrypted storage. [H2] Validates region.
#[tauri::command]
async fn set_api_key(
    key: String,
    db: tauri::State<'_, Arc<Database>>,
    api_state: tauri::State<'_, Arc<AsyncMutex<ApiState>>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // [H2] Validate key format
    let key = key.trim().to_string();
    if key.is_empty() {
        return Err("API key cannot be empty".to_string());
    }
    if !key.starts_with("RGAPI-") && key.len() > 10 {
        return Err("Invalid API key format. Keys start with RGAPI-".to_string());
    }

    // [C1] Store encrypted
    db.set_api_key(&key).map_err(|e| safe_err("Save API key", e))?;

    // [H2] Validate region
    let platform = db
        .get_state("region")
        .map_err(|e| safe_err("Get region", e))?
        .unwrap_or_else(|| "la1".to_string());
    validate_region(&platform)?;

    let api_client = Arc::new(RiotApiClient::new(key, platform));
    let fetcher = Arc::new(MatchFetcher::new(api_client, db.inner().clone(), app_handle));

    // [H4] Direct await instead of block_on
    let mut state = api_state.lock().await;
    state.fetcher = Some(fetcher);

    tracing::info!("API key configured");
    Ok(())
}

/// [H4] Now async. [H2-H3] Validates region against allowlist.
#[tauri::command]
async fn set_region(
    region: String,
    db: tauri::State<'_, Arc<Database>>,
    api_state: tauri::State<'_, Arc<AsyncMutex<ApiState>>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // [H2-H3] Validate region
    validate_region(&region)?;

    db.set_state("region", &region).map_err(|e| safe_err("Save region", e))?;

    // Re-initialize if API key exists [C1] using encrypted getter
    if let Ok(Some(api_key)) = db.get_api_key() {
        let api_client = Arc::new(RiotApiClient::new(api_key, region));
        let fetcher = Arc::new(MatchFetcher::new(api_client, db.inner().clone(), app_handle));

        // [H4] Direct await
        let mut state = api_state.lock().await;
        state.fetcher = Some(fetcher);
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
    validate_puuid(&puuid)?; // [M4]
    let count = count.clamp(1, 100); // [M2]
    let offset = offset.max(0); // [M2]

    let matches = db.get_match_history(&puuid, count, offset).map_err(|e| safe_err("Match history", e))?;
    let total = db.get_match_count(&puuid).map_err(|e| safe_err("Match count", e))?;
    Ok(serde_json::json!({
        "matches": matches,
        "total": total,
    }))
}

#[tauri::command]
async fn trigger_backfill(
    puuid: String,
    api_state: tauri::State<'_, Arc<AsyncMutex<ApiState>>>,
    lcu: tauri::State<'_, Arc<LcuManager>>,
    db: tauri::State<'_, Arc<Database>>,
    app_handle: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    validate_puuid(&puuid)?; // [M4]

    let fetcher = {
        let state = api_state.lock().await;
        state.fetcher.clone()
    };

    if let Some(fetcher) = fetcher {
        if let Err(e) = fetcher.update_static_data().await {
            tracing::warn!("Failed to update static data: {e}");
        }

        let count = fetcher
            .backfill(&puuid, 20)
            .await
            .map_err(|e| safe_err("Backfill", e))?;

        return Ok(serde_json::json!({ "fetched": count, "source": "riot_api" }));
    }

    let lcu_client = lcu.get_client().ok_or("Not connected to League client")?;

    let _ = app_handle.emit("backfill-progress", serde_json::json!({"current": 0, "total": 0, "match_id": "Fetching from League client..."}));

    let matches = lcu_client
        .get_match_history(&puuid, 20)
        .await
        .map_err(|e| safe_err("LCU match fetch", e))?;

    let mut stored = 0;
    for m in &matches {
        if db.has_match(&m.match_id).unwrap_or(true) {
            continue;
        }
        if let Err(e) = db.store_match(
            &m.match_id, m.game_creation, m.game_duration,
            &m.game_mode, m.queue_id, None, &m.raw_json, &m.participants,
        ) {
            tracing::warn!("Failed to store LCU match {}: {e}", m.match_id);
            continue;
        }
        stored += 1;
    }

    let _ = app_handle.emit("backfill-complete", serde_json::json!({"fetched": stored}));
    tracing::info!("Stored {stored} matches from LCU");
    Ok(serde_json::json!({ "fetched": stored, "source": "lcu" }))
}

#[tauri::command]
fn has_api_key(db: tauri::State<'_, Arc<Database>>) -> bool {
    db.get_api_key() // [C1] Uses encrypted getter
        .ok()
        .flatten()
        .map(|k| !k.is_empty())
        .unwrap_or(false)
}

#[tauri::command]
async fn get_live_game_state(
    live_state: tauri::State<'_, Arc<AsyncMutex<Option<game_client::state::LiveGameState>>>>,
) -> Result<serde_json::Value, String> {
    let state = live_state.lock().await.clone(); // [H4] proper async
    match state {
        Some(s) => serde_json::to_value(&s).map_err(|e| safe_err("Live game state", e)),
        None => Ok(serde_json::Value::Null),
    }
}

#[tauri::command]
fn get_champ_select_session(
    lcu: tauri::State<'_, Arc<LcuManager>>,
) -> Result<serde_json::Value, String> {
    match lcu.get_champ_select() {
        Some(session) => serde_json::to_value(&session).map_err(|e| safe_err("Champ select", e)),
        None => Ok(serde_json::Value::Null),
    }
}

#[tauri::command]
fn get_draft_recommendations(
    puuid: String,
    lcu: tauri::State<'_, Arc<LcuManager>>,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<serde_json::Value, String> {
    validate_puuid(&puuid)?; // [M4]
    let session = lcu.get_champ_select().ok_or("Not in champion select")?;
    let recs = analysis::draft::get_recommendations(&session, db.inner(), &puuid);
    serde_json::to_value(&recs).map_err(|e| safe_err("Draft recommendations", e))
}

#[tauri::command]
fn get_champion_pool(
    puuid: String,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<serde_json::Value, String> {
    validate_puuid(&puuid)?; // [M4]
    let pool = db.get_champion_pool(&puuid, 1).map_err(|e| safe_err("Champion pool", e))?;
    serde_json::to_value(&pool).map_err(|e| safe_err("Champion pool", e))
}

#[tauri::command]
fn get_post_game_analysis(
    match_id: String,
    puuid: String,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<serde_json::Value, String> {
    validate_puuid(&puuid)?; // [M4]

    if let Ok(Some(cached)) = db.get_post_game_analysis(&match_id) {
        return serde_json::from_str(&cached).map_err(|e| safe_err("Parse analysis", e));
    }

    let match_json = db.get_match_json(&match_id).map_err(|e| safe_err("Get match", e))?
        .ok_or("Match not found")?;
    let timeline_json = db.get_timeline_json(&match_id).map_err(|e| safe_err("Get timeline", e))?
        .ok_or("Timeline not found")?;

    let match_data: riot_api::types::RiotMatch = serde_json::from_str(&match_json)
        .map_err(|e| safe_err("Parse match", e))?;
    let timeline: riot_api::types::MatchTimeline = serde_json::from_str(&timeline_json)
        .map_err(|e| safe_err("Parse timeline", e))?;

    if !db.has_features(&match_id, &puuid).unwrap_or(true) {
        if let Some(features) = analysis::patterns::extract_features(&match_data, &timeline, &puuid) {
            let fj = serde_json::to_string(&features).unwrap_or_default();
            let _ = db.store_features(&match_id, &puuid, &fj);
        }
    }

    let analysis = analysis::post_game::analyze(&match_data, &timeline, &puuid, db.inner())
        .ok_or("Failed to generate analysis")?;

    let analysis_json = serde_json::to_string(&analysis).unwrap_or_default();
    let _ = db.store_post_game_analysis(&match_id, &puuid, &analysis_json);

    serde_json::to_value(&analysis).map_err(|e| safe_err("Serialize analysis", e))
}

#[tauri::command]
fn get_detected_patterns(
    puuid: String,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<serde_json::Value, String> {
    validate_puuid(&puuid)?; // [M4]
    let patterns = db.get_patterns(&puuid).map_err(|e| safe_err("Get patterns", e))?;
    Ok(serde_json::Value::Array(patterns))
}

#[tauri::command]
fn run_pattern_analysis(
    puuid: String,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<serde_json::Value, String> {
    validate_puuid(&puuid)?; // [M4]

    let matches = db.get_match_history(&puuid, 200, 0).map_err(|e| safe_err("Get matches", e))?;
    let mut extracted = 0;

    for m in &matches {
        if db.has_features(&m.match_id, &puuid).unwrap_or(true) {
            continue;
        }
        let match_json = match db.get_match_json(&m.match_id) {
            Ok(Some(j)) => j,
            _ => continue,
        };
        let timeline_json = match db.get_timeline_json(&m.match_id) {
            Ok(Some(j)) => j,
            _ => continue,
        };
        let Ok(match_data) = serde_json::from_str::<riot_api::types::RiotMatch>(&match_json) else { continue };
        let Ok(timeline) = serde_json::from_str::<riot_api::types::MatchTimeline>(&timeline_json) else { continue };

        if let Some(features) = analysis::patterns::extract_features(&match_data, &timeline, &puuid) {
            let fj = serde_json::to_string(&features).unwrap_or_default();
            let _ = db.store_features(&m.match_id, &puuid, &fj);
            extracted += 1;
        }
    }

    let patterns = analysis::patterns::detect_patterns(db.inner(), &puuid);

    Ok(serde_json::json!({
        "features_extracted": extracted,
        "patterns_detected": patterns.len(),
        "patterns": patterns,
    }))
}

#[tauri::command]
fn get_weekly_metrics(
    puuid: String,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<serde_json::Value, String> {
    validate_puuid(&puuid)?; // [M4]
    let metrics = analysis::improvement::compute_metrics(db.inner(), &puuid);
    serde_json::to_value(&metrics).map_err(|e| safe_err("Weekly metrics", e))
}

#[tauri::command]
fn get_improvement_goals(
    puuid: String,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<serde_json::Value, String> {
    validate_puuid(&puuid)?; // [M4]
    let goals = db.get_goals(&puuid).map_err(|e| safe_err("Get goals", e))?;
    Ok(serde_json::Value::Array(goals))
}

#[tauri::command]
fn create_improvement_goal(
    puuid: String,
    name: String,
    metric_key: String,
    target_value: Option<f64>,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<serde_json::Value, String> {
    validate_puuid(&puuid)?; // [M4]
    let id = db.create_goal(&puuid, &name, None, &metric_key, target_value, None)
        .map_err(|e| safe_err("Create goal", e))?;
    Ok(serde_json::json!({ "id": id }))
}

// ── App Setup ─────────────────────────────────────────────

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

            let db = Arc::new(Database::new(&app_handle)?);
            db.run_migrations()?;
            tracing::info!("Database initialized");
            app.manage(db.clone());

            // Data Dragon (no API key needed)
            let ddragon_db = db.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = riot_api::ddragon::update_static_data(&ddragon_db).await {
                    tracing::warn!("Failed to fetch Data Dragon: {e}");
                }
            });

            // API state
            let api_state = Arc::new(AsyncMutex::new(ApiState { fetcher: None }));

            // [C1] Restore API key using encrypted getter
            if let Ok(Some(api_key)) = db.get_api_key() {
                let platform = db
                    .get_state("region")
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| "la1".to_string());

                // [H2] Validate restored region
                if VALID_REGIONS.contains(&platform.as_str()) {
                    let api_client = Arc::new(RiotApiClient::new(api_key, platform));
                    let fetcher = Arc::new(MatchFetcher::new(
                        api_client, db.clone(), app_handle.clone(),
                    ));
                    tauri::async_runtime::block_on(async {
                        let mut state = api_state.lock().await;
                        state.fetcher = Some(fetcher);
                    });
                    tracing::info!("Restored API key from database");
                }
            }

            app.manage(api_state);

            // Live game state
            let live_game_state: Arc<AsyncMutex<Option<game_client::state::LiveGameState>>> =
                Arc::new(AsyncMutex::new(None));
            app.manage(live_game_state.clone());

            // LCU Manager
            let lcu_manager = Arc::new(LcuManager::new(app_handle.clone()));
            app.manage(lcu_manager.clone());

            // Game lifecycle listener
            let mut lcu_rx = lcu_manager.subscribe();
            let gc_app_handle = app_handle.clone();
            let gc_live_state = live_game_state.clone();
            let gc_db = db.clone();
            let gc_lcu = lcu_manager.clone();

            tauri::async_runtime::spawn(async move {
                let mut poller_handle: Option<tokio::task::JoinHandle<()>> = None;

                loop {
                    match lcu_rx.recv().await {
                        Ok(lcu::types::LcuEvent::GameFlowChanged { phase }) => {
                            match phase {
                                lcu::types::GameFlowPhase::InProgress => {
                                    if poller_handle.is_none() {
                                        tracing::info!("Game started, spawning live game poller");
                                        let handle = gc_app_handle.clone();
                                        let state_ref = gc_live_state.clone();
                                        let poller_db = gc_db.clone();

                                        let puuid = gc_lcu
                                            .get_state()
                                            .summoner
                                            .map(|s| s.puuid.clone())
                                            .unwrap_or_default();

                                        poller_handle = Some(tokio::spawn(async move {
                                            let mut poller =
                                                game_client::poller::GameClientPoller::new(
                                                    handle, Some(poller_db.clone()), puuid.clone(),
                                                );
                                            let session_id = poller.session_id().to_string();

                                            if let Err(e) = poller.run().await {
                                                tracing::warn!("Game poller error: {e}");
                                            }

                                            let final_state = poller.get_state();
                                            if !puuid.is_empty() && final_state.game_time > 300.0 {
                                                let local = final_state.my_team.iter().find(|p| p.is_local_player);
                                                if let Some(local) = local {
                                                    if let Some(features) = analysis::live_timeline::extract_features_from_session(
                                                        &poller_db, &session_id, &puuid,
                                                        &local.champion, 0, "", false,
                                                        final_state.game_time,
                                                    ) {
                                                        let fj = serde_json::to_string(&features).unwrap_or_default();
                                                        let match_id = format!("live_{session_id}");
                                                        let _ = poller_db.store_features(&match_id, &puuid, &fj);
                                                        tracing::info!("Extracted features from live session {session_id}");
                                                    }
                                                }
                                            }

                                            {
                                                let mut lock = state_ref.lock().await;
                                                *lock = Some(final_state);
                                            }
                                            tracing::info!("Game poller stopped");
                                        }));
                                    }
                                }
                                lcu::types::GameFlowPhase::None
                                | lcu::types::GameFlowPhase::Lobby
                                | lcu::types::GameFlowPhase::EndOfGame => {
                                    if let Some(handle) = poller_handle.take() {
                                        handle.abort();
                                        tracing::info!("Game poller aborted (phase: {phase})");
                                    }
                                    if phase != lcu::types::GameFlowPhase::EndOfGame {
                                        let mut lock = gc_live_state.lock().await;
                                        *lock = None;
                                    }
                                }
                                _ => {}
                            }
                        }
                        Ok(lcu::types::LcuEvent::Disconnected) => {
                            if let Some(handle) = poller_handle.take() {
                                handle.abort();
                            }
                            let mut lock = gc_live_state.lock().await;
                            *lock = None;
                        }
                        Ok(_) => {}
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("Game lifecycle listener lagged by {n}");
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
            });

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
            get_champion_map,
            set_api_key,
            set_region,
            get_match_history,
            trigger_backfill,
            get_champ_select_session,
            get_draft_recommendations,
            get_champion_pool,
            get_live_game_state,
            has_api_key,
            get_post_game_analysis,
            get_detected_patterns,
            run_pattern_analysis,
            get_weekly_metrics,
            get_improvement_goals,
            create_improvement_goal,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Sentinel");
}
