use super::tls;
use super::types::{GameFlowPhase, LcuSummoner, LockfileData};
use anyhow::{Context, Result};

/// REST client for the League Client Update API
#[derive(Clone)]
pub struct LcuClient {
    client: reqwest::Client,
    base_url: String,
}

impl LcuClient {
    /// Create a new LCU client from lockfile credentials
    pub fn new(lockfile: &LockfileData) -> Self {
        let client = tls::build_lcu_http_client(&lockfile.password);
        let base_url = format!("https://127.0.0.1:{}", lockfile.port);
        Self { client, base_url }
    }

    /// Make a GET request to an LCU endpoint
    async fn get<T: serde::de::DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, endpoint);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("LCU request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("LCU returned {status}: {body}");
        }

        resp.json::<T>().await.context("Failed to parse LCU response")
    }

    /// Get the currently logged-in summoner
    pub async fn get_current_summoner(&self) -> Result<LcuSummoner> {
        self.get("/lol-summoner/v1/current-summoner").await
    }

    /// Get the current game flow phase
    pub async fn get_gameflow_phase(&self) -> Result<GameFlowPhase> {
        let phase_str: String = self.get("/lol-gameflow/v1/gameflow-phase").await?;
        Ok(GameFlowPhase::from_str_lossy(&phase_str))
    }

    /// Check if the client is reachable
    pub async fn is_alive(&self) -> bool {
        self.client
            .get(format!("{}/lol-gameflow/v1/gameflow-phase", self.base_url))
            .send()
            .await
            .is_ok()
    }
}
