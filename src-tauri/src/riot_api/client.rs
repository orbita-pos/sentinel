use anyhow::{Context, Result};

use super::rate_limiter::RateLimiter;
use super::types::{self, LeagueEntry, MatchTimeline, RiotMatch};

/// Rate-limited Riot API HTTP client
pub struct RiotApiClient {
    http: reqwest::Client,
    rate_limiter: RateLimiter,
    api_key: String,
    platform: String, // e.g., "na1"
    region: String,   // e.g., "americas"
}

impl RiotApiClient {
    pub fn new(api_key: String, platform: String) -> Self {
        let region = types::platform_to_region(&platform).to_string();
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http,
            rate_limiter: RateLimiter::dev(),
            api_key,
            platform,
            region,
        }
    }

    /// Make a rate-limited GET request with one retry on 429
    async fn get<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T> {
        for attempt in 0..2 {
            self.rate_limiter.acquire().await;

            let resp = self
                .http
                .get(url)
                .header("X-Riot-Token", &self.api_key)
                .send()
                .await
                .context("Riot API request failed")?;

            let status = resp.status();
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS && attempt == 0 {
                let retry_after = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(5);
                tracing::warn!("Rate limited, retrying in {retry_after}s");
                tokio::time::sleep(std::time::Duration::from_secs(retry_after)).await;
                continue;
            }

            if !status.is_success() {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Riot API {status}: {body}");
            }

            return resp.json::<T>().await.context("Failed to parse Riot API response");
        }
        anyhow::bail!("Riot API request failed after retries")
    }

    /// Get raw JSON response with one retry on 429
    async fn get_raw(&self, url: &str) -> Result<String> {
        for attempt in 0..2 {
            self.rate_limiter.acquire().await;

            let resp = self
                .http
                .get(url)
                .header("X-Riot-Token", &self.api_key)
                .send()
                .await
                .context("Riot API request failed")?;

            let status = resp.status();
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS && attempt == 0 {
                let retry_after = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(5);
                tracing::warn!("Rate limited, retrying in {retry_after}s");
                tokio::time::sleep(std::time::Duration::from_secs(retry_after)).await;
                continue;
            }

            if !status.is_success() {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Riot API {status}: {body}");
            }

            return resp.text().await.context("Failed to read response body");
        }
        anyhow::bail!("Riot API request failed after retries")
    }

    // ── Match V5 endpoints ──────────────────────────────

    /// Get recent match IDs for a player
    pub async fn get_match_ids(&self, puuid: &str, count: i32) -> Result<Vec<String>> {
        let url = format!(
            "https://{}.api.riotgames.com/lol/match/v5/matches/by-puuid/{}/ids?count={}",
            self.region, puuid, count
        );
        self.get(&url).await
    }

    /// Get full match data (parsed + raw JSON)
    pub async fn get_match(&self, match_id: &str) -> Result<(RiotMatch, String)> {
        let url = format!(
            "https://{}.api.riotgames.com/lol/match/v5/matches/{}",
            self.region, match_id
        );
        let raw = self.get_raw(&url).await?;
        let parsed: RiotMatch = serde_json::from_str(&raw)?;
        Ok((parsed, raw))
    }

    /// Get match timeline (parsed + raw JSON)
    pub async fn get_timeline(&self, match_id: &str) -> Result<(MatchTimeline, String)> {
        let url = format!(
            "https://{}.api.riotgames.com/lol/match/v5/matches/{}/timeline",
            self.region, match_id
        );
        let raw = self.get_raw(&url).await?;
        let parsed: MatchTimeline = serde_json::from_str(&raw)?;
        Ok((parsed, raw))
    }

    // ── League / Ranked ─────────────────────────────────

    /// Get ranked entries for a summoner
    #[allow(dead_code)]
    pub async fn get_league_entries(&self, summoner_id: &str) -> Result<Vec<LeagueEntry>> {
        let url = format!(
            "https://{}.api.riotgames.com/lol/league/v4/entries/by-summoner/{}",
            self.platform, summoner_id
        );
        self.get(&url).await
    }

    // ── Data Dragon ─────────────────────────────────────

    /// Get the latest game version from Data Dragon
    pub async fn get_latest_version(&self) -> Result<String> {
        let versions: Vec<String> = self
            .http
            .get("https://ddragon.leagueoflegends.com/api/versions.json")
            .send()
            .await?
            .json()
            .await?;
        versions
            .into_iter()
            .next()
            .context("No versions available")
    }

    /// Get champion data from Data Dragon
    pub async fn get_champions(&self, version: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://ddragon.leagueoflegends.com/cdn/{version}/data/en_US/champion.json"
        );
        self.http
            .get(&url)
            .send()
            .await?
            .json()
            .await
            .context("Failed to fetch champions")
    }

    /// Get item data from Data Dragon
    pub async fn get_items(&self, version: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://ddragon.leagueoflegends.com/cdn/{version}/data/en_US/item.json"
        );
        self.http
            .get(&url)
            .send()
            .await?
            .json()
            .await
            .context("Failed to fetch items")
    }
}
