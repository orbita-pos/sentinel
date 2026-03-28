pub mod champ_select;
pub mod connector;
pub mod lockfile;
pub mod runes;
pub mod tls;
pub mod types;
pub mod websocket;

use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter};
use tokio::sync::broadcast;

use self::champ_select::ChampSelectSession;
use self::connector::LcuClient;
use self::types::{ConnectionState, GameFlowPhase, LcuEvent, LcuSummoner};

/// Manages the lifecycle of the LCU connection.
///
/// Continuously watches for the League client, connects when found,
/// maintains the WebSocket subscription, and reconnects on disconnect.
pub struct LcuManager {
    app_handle: AppHandle,
    event_tx: broadcast::Sender<LcuEvent>,
    state: Arc<Mutex<ConnectionState>>,
    champ_select_session: Arc<Mutex<Option<ChampSelectSession>>>,
    current_client: Arc<Mutex<Option<LcuClient>>>,
}

impl LcuManager {
    pub fn new(app_handle: AppHandle) -> Self {
        let (event_tx, _) = broadcast::channel(64);
        let state = Arc::new(Mutex::new(ConnectionState {
            status: "disconnected".to_string(),
            summoner: None,
            game_phase: "None".to_string(),
        }));
        Self {
            app_handle,
            event_tx,
            state,
            champ_select_session: Arc::new(Mutex::new(None)),
            current_client: Arc::new(Mutex::new(None)),
        }
    }

    /// Get a receiver for LCU events
    pub fn subscribe(&self) -> broadcast::Receiver<LcuEvent> {
        self.event_tx.subscribe()
    }

    /// Get current connection state (for Tauri commands)
    pub fn get_state(&self) -> ConnectionState {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Get current champ select session (if in ChampSelect phase)
    pub fn get_champ_select(&self) -> Option<ChampSelectSession> {
        self.champ_select_session.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Get the current LCU client (if connected)
    pub fn get_client(&self) -> Option<LcuClient> {
        self.current_client.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Main loop: detect client → connect → maintain → reconnect
    pub async fn run(&self) {
        loop {
            // Phase 1: Wait for League client
            tracing::info!("Watching for League client...");
            self.update_status("disconnected", None, "None");

            let lockfile = loop {
                if let Some(data) = lockfile::detect() {
                    tracing::info!(
                        "League client detected on port {} (pid {})",
                        data.port,
                        data.pid
                    );
                    break data;
                }
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            };

            // Phase 2: Connect and fetch initial data
            self.update_status("connecting", None, "None");

            let client = LcuClient::new(&lockfile);

            // Store the client for external access (match history, etc.)
            if let Ok(mut c) = self.current_client.lock() {
                *c = Some(client.clone());
            }

            // Fetch summoner and initial phase
            let summoner = match client.get_current_summoner().await {
                Ok(s) => {
                    tracing::info!("Connected as {}#{}", s.game_name, s.tag_line);
                    s
                }
                Err(e) => {
                    tracing::warn!("Failed to get summoner: {e}");
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    continue;
                }
            };

            let phase = client
                .get_gameflow_phase()
                .await
                .unwrap_or(GameFlowPhase::None);

            // Emit connected event
            self.update_status("connected", Some(summoner.clone()), phase.as_str());
            self.emit(LcuEvent::Connected {
                summoner: summoner.clone(),
            });
            self.emit(LcuEvent::GameFlowChanged {
                phase: phase.clone(),
            });

            // Phase 3: Run WebSocket for real-time events
            tracing::info!("Starting WebSocket listener...");

            // Clone tx for the websocket task
            let ws_tx = self.event_tx.clone();
            let ws_lockfile = lockfile.clone();

            // Also listen to our own events to update internal state
            let mut state_rx = self.event_tx.subscribe();
            let state_ref = self.state.clone();
            let cs_ref = self.champ_select_session.clone();
            let app_handle = self.app_handle.clone();

            // Spawn state updater that forwards events to frontend
            let state_task = tokio::spawn(async move {
                loop {
                    match state_rx.recv().await {
                        Ok(event) => {
                            // Update internal state
                            match &event {
                                LcuEvent::GameFlowChanged { phase } => {
                                    if let Ok(mut state) = state_ref.lock() {
                                        state.game_phase = phase.as_str().to_string();
                                    }
                                    // Clear champ select when leaving that phase
                                    if *phase != GameFlowPhase::ChampSelect {
                                        if let Ok(mut cs) = cs_ref.lock() {
                                            *cs = None;
                                        }
                                    }
                                }
                                LcuEvent::ChampSelectUpdate { data } => {
                                    // Parse and store champ select session
                                    if let Some(session) = champ_select::parse_session(data, None) {
                                        // Emit parsed session to frontend
                                        let _ = app_handle.emit("champ-select-update", &session);
                                        if let Ok(mut cs) = cs_ref.lock() {
                                            *cs = Some(session);
                                        }
                                    }
                                }
                                LcuEvent::Disconnected => {
                                    if let Ok(mut state) = state_ref.lock() {
                                        state.status = "disconnected".to_string();
                                        state.summoner = None;
                                        state.game_phase = "None".to_string();
                                    }
                                    if let Ok(mut cs) = cs_ref.lock() {
                                        *cs = None;
                                    }
                                }
                                _ => {}
                            }
                            // Forward raw event to frontend
                            let _ = app_handle.emit("lcu-event", &event);
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("Event listener lagged by {n} messages");
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
            });

            // Run WebSocket (blocks until disconnect)
            if let Err(e) = websocket::run_websocket(&ws_lockfile, ws_tx).await {
                tracing::warn!("WebSocket disconnected: {e}");
            }

            // Cleanup
            state_task.abort();
            if let Ok(mut c) = self.current_client.lock() {
                *c = None;
            }
            self.emit(LcuEvent::Disconnected);
            self.update_status("disconnected", None, "None");
            tracing::info!("League client disconnected, will retry...");

            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }

    fn emit(&self, event: LcuEvent) {
        let _ = self.event_tx.send(event.clone());
        let _ = self.app_handle.emit("lcu-event", &event);
    }

    fn update_status(&self, status: &str, summoner: Option<LcuSummoner>, phase: &str) {
        if let Ok(mut state) = self.state.lock() {
            state.status = status.to_string();
            state.summoner = summoner;
            state.game_phase = phase.to_string();
        }
        // Emit a synthetic event for the frontend
        let _ = self.app_handle.emit(
            "lcu-status",
            &serde_json::json!({
                "status": status,
                "game_phase": phase,
            }),
        );
    }
}
