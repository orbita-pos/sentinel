use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use tauri::{AppHandle, Emitter};

use super::state::LiveGameState;
use super::types::{AllPlayer, ActivePlayer, EventData, GameStats};

/// Polls the Game Client API at 1Hz and emits state updates
pub struct GameClientPoller {
    http: reqwest::Client,
    app_handle: AppHandle,
    state: Arc<Mutex<LiveGameState>>,
    local_player_name: String,
}

const BASE_URL: &str = "https://127.0.0.1:2999/liveclientdata";

impl GameClientPoller {
    pub fn new(app_handle: AppHandle) -> Self {
        let http = reqwest::Client::builder()
            .danger_accept_invalid_certs(true) // Game client uses self-signed cert
            .timeout(Duration::from_secs(3))
            .build()
            .expect("Failed to build game client HTTP client");

        Self {
            http,
            app_handle,
            state: Arc::new(Mutex::new(LiveGameState::default())),
            local_player_name: String::new(),
        }
    }

    /// Get current state (for Tauri commands)
    pub fn get_state(&self) -> LiveGameState {
        self.state.lock().unwrap().clone()
    }

    /// Run the polling loop. Returns when the game ends or connection is lost.
    pub async fn run(&mut self) -> Result<()> {
        tracing::info!("Game client poller starting...");

        // Wait for the game client API to become available
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

        // Fetch active player name once
        if let Ok(active) = self.fetch::<ActivePlayer>("/activeplayer").await {
            self.local_player_name = active.riot_id_game_name;
            tracing::info!("Local player: {}", self.local_player_name);
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
                        || err_str.contains("os error 10061") // Windows WSAECONNREFUSED
                        || err_str.contains("error sending request")
                    {
                        tracing::info!("Game client disconnected (game ended)");
                        break;
                    }
                    tracing::debug!("Poll error: {e}");
                }
            }
        }

        Ok(())
    }

    async fn poll_once(&mut self) -> Result<()> {
        // Fetch all endpoints concurrently
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

        // Emit state to frontend
        let state = self.state.lock().unwrap().clone();

        // Emit new power spikes individually
        if state.power_spikes.len() > prev_spike_count {
            for spike in &state.power_spikes[prev_spike_count..] {
                let _ = self.app_handle.emit("power-spike", spike);
            }
        }

        let _ = self.app_handle.emit("live-game-update", &state);

        Ok(())
    }

    async fn fetch<T: serde::de::DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{BASE_URL}{endpoint}");
        let resp = self.http.get(&url).send().await?;
        let data = resp.json::<T>().await?;
        Ok(data)
    }
}
