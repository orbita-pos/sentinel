use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use tauri::{AppHandle, Emitter};

use super::state::LiveGameState;
use super::types::{AllPlayer, ActivePlayer, EventData, GameStats};
use crate::database::{Database, LiveSnapshotRow};

/// Polls the Game Client API at 1Hz and emits state updates.
/// Also records snapshots to SQLite for timeline reconstruction.
pub struct GameClientPoller {
    http: reqwest::Client,
    app_handle: AppHandle,
    state: Arc<Mutex<LiveGameState>>,
    local_player_name: String,
    // Live capture
    db: Option<Arc<Database>>,
    session_id: String,
    last_snapshot_time: f64,
    last_event_count: usize,
    puuid: String,
}

const BASE_URL: &str = "https://127.0.0.1:2999/liveclientdata";
const SNAPSHOT_INTERVAL: f64 = 10.0; // Record every 10 seconds

impl GameClientPoller {
    pub fn new(app_handle: AppHandle, db: Option<Arc<Database>>, puuid: String) -> Self {
        let http = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(3))
            .build()
            .expect("Failed to build game client HTTP client");

        let session_id = format!("live_{}", chrono::Utc::now().timestamp_millis());

        Self {
            http,
            app_handle,
            state: Arc::new(Mutex::new(LiveGameState::default())),
            local_player_name: String::new(),
            db,
            session_id,
            last_snapshot_time: -SNAPSHOT_INTERVAL, // Force first snapshot
            last_event_count: 0,
            puuid,
        }
    }

    /// Get current state
    pub fn get_state(&self) -> LiveGameState {
        self.state.lock().unwrap().clone()
    }

    /// Get the session ID for this game
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Run the polling loop. Returns when the game ends.
    pub async fn run(&mut self) -> Result<()> {
        tracing::info!("Game client poller starting (session: {})", self.session_id);

        // Wait for game client API
        let mut retries = 0;
        loop {
            match self.http.get(format!("{BASE_URL}/gamestats")).send().await {
                Ok(_) => break,
                Err(_) if retries < 30 => {
                    retries += 1;
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
                Err(e) => {
                    tracing::warn!("Game client not available after {retries} retries: {e}");
                    return Ok(());
                }
            }
        }

        tracing::info!("Game client API available, starting 1Hz polling");

        // Fetch active player name
        if let Ok(active) = self.fetch::<ActivePlayer>("/activeplayer").await {
            self.local_player_name = active.riot_id_game_name;
            tracing::info!("Local player: {}", self.local_player_name);
        }

        // Create DB session
        if let Some(db) = &self.db {
            let _ = db.create_live_session(&self.session_id, &self.puuid);
        }

        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;

            match self.poll_once().await {
                Ok(()) => {}
                Err(e) => {
                    let err_str = e.to_string();
                    if err_str.contains("connection refused")
                        || err_str.contains("Connection refused")
                        || err_str.contains("os error 10061")
                        || err_str.contains("error sending request")
                    {
                        tracing::info!("Game client disconnected (game ended)");
                        break;
                    }
                    tracing::debug!("Poll error: {e}");
                }
            }
        }

        // Finalize session
        self.finalize_session();

        Ok(())
    }

    async fn poll_once(&mut self) -> Result<()> {
        let (players_res, events_res, stats_res) = tokio::join!(
            self.fetch::<Vec<AllPlayer>>("/playerlist"),
            self.fetch::<EventData>("/eventdata"),
            self.fetch::<GameStats>("/gamestats"),
        );

        let players = players_res?;
        let events = events_res.unwrap_or_default();
        let stats = stats_res?;

        // Collect new power spikes before updating
        let prev_spike_count;
        {
            let state = self.state.lock().unwrap();
            prev_spike_count = state.power_spikes.len();
        }

        // Update composite state
        {
            let mut state = self.state.lock().unwrap();
            state.update(&players, &events, &stats, &self.local_player_name);
        }

        // ── Record snapshot to DB every SNAPSHOT_INTERVAL seconds ──
        if stats.game_time - self.last_snapshot_time >= SNAPSHOT_INTERVAL {
            self.record_snapshot(&players, &stats);
            self.last_snapshot_time = stats.game_time;
        }

        // ── Record new events ──
        let current_event_count = events.events.len();
        if current_event_count > self.last_event_count {
            self.record_new_events(&events, self.last_event_count);
            self.last_event_count = current_event_count;
        }

        // Emit state to frontend
        let state = self.state.lock().unwrap().clone();

        if state.power_spikes.len() > prev_spike_count {
            for spike in &state.power_spikes[prev_spike_count..] {
                let _ = self.app_handle.emit("power-spike", spike);
            }
        }

        let _ = self.app_handle.emit("live-game-update", &state);

        Ok(())
    }

    /// Record a snapshot of all players to DB
    fn record_snapshot(&self, players: &[AllPlayer], stats: &GameStats) {
        let Some(db) = &self.db else { return };

        let snapshots: Vec<LiveSnapshotRow> = players
            .iter()
            .map(|p| {
                let item_gold: i64 = p.items.iter().map(|i| i.price * i.count).sum();
                LiveSnapshotRow {
                    game_time: stats.game_time,
                    player_name: p.riot_id_game_name.clone(),
                    champion: p.champion_name.clone(),
                    team: p.team.clone(),
                    level: p.level,
                    kills: p.scores.kills,
                    deaths: p.scores.deaths,
                    assists: p.scores.assists,
                    cs: p.scores.creep_score,
                    ward_score: p.scores.ward_score,
                    item_gold,
                    is_local: p.riot_id_game_name == self.local_player_name,
                }
            })
            .collect();

        if let Err(e) = db.store_live_snapshots(&self.session_id, &snapshots) {
            tracing::debug!("Failed to store snapshot: {e}");
        }
    }

    /// Record new game events to DB
    fn record_new_events(&self, events: &EventData, from_index: usize) {
        let Some(db) = &self.db else { return };

        for event in events.events.iter().skip(from_index) {
            let name = &event.event_name;
            // Only record notable events
            let notable = matches!(
                name.as_str(),
                "ChampionKill" | "DragonKill" | "BaronKill" | "HeraldKill"
                    | "TurretKilled" | "InhibKilled" | "Multikill" | "Ace"
            );
            if notable {
                let _ = db.store_live_event(
                    &self.session_id,
                    event.event_time,
                    name,
                    &format!("{name} at {:.0}s", event.event_time),
                );
            }
        }
    }

    /// Finalize the session with game results from final state
    fn finalize_session(&self) {
        let Some(db) = &self.db else { return };
        let state = self.state.lock().unwrap();

        let local = state
            .my_team
            .iter()
            .find(|p| p.is_local_player);

        let champion = local.map(|p| p.champion.as_str()).unwrap_or("");
        // We can't definitively know win/loss from the Game Client API alone
        // It will be updated later when the EndOfGame event arrives
        let _ = db.finalize_live_session(
            &self.session_id,
            None, // match_id set later
            champion,
            &state.game_mode,
            state.game_time,
            None, // win set later
        );

        tracing::info!(
            "Live session {} finalized: {:.0}s, champion: {champion}",
            self.session_id,
            state.game_time,
        );
    }

    async fn fetch<T: serde::de::DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{BASE_URL}{endpoint}");
        let resp = self.http.get(&url).send().await?;
        let data = resp.json::<T>().await?;
        Ok(data)
    }
}
