use std::sync::Arc;

use anyhow::Result;
use tauri::{AppHandle, Emitter};

use super::client::RiotApiClient;
use crate::database::{Database, MatchParticipantRow};

/// Fetches match data from the Riot API and stores it locally.
pub struct MatchFetcher {
    api: Arc<RiotApiClient>,
    db: Arc<Database>,
    app_handle: AppHandle,
}

impl MatchFetcher {
    pub fn new(api: Arc<RiotApiClient>, db: Arc<Database>, app_handle: AppHandle) -> Self {
        Self { api, db, app_handle }
    }

    /// Fetch recent matches for a player, return IDs not already in the DB
    pub async fn get_new_match_ids(&self, puuid: &str, count: i32) -> Result<Vec<String>> {
        let all_ids = self.api.get_match_ids(puuid, count).await?;
        let new_ids: Vec<String> = all_ids
            .into_iter()
            .filter(|id| !self.db.has_match(id).unwrap_or(true))
            .collect();
        tracing::debug!("Found {} new matches", new_ids.len());
        Ok(new_ids)
    }

    /// Fetch a single match + timeline and store in the database
    pub async fn fetch_and_store_match(&self, match_id: &str) -> Result<()> {
        if self.db.has_match(match_id).unwrap_or(true) {
            return Ok(()); // Already stored
        }

        tracing::info!("Fetching match {match_id}");

        // Fetch match data
        let (riot_match, raw_json) = self.api.get_match(match_id).await?;

        // Build participant rows
        let participants: Vec<MatchParticipantRow> = riot_match
            .info
            .participants
            .iter()
            .map(|p| MatchParticipantRow {
                participant_id: p.participant_id,
                puuid: p.puuid.clone(),
                champion_id: p.champion_id,
                champion_name: p.champion_name.clone(),
                team_id: p.team_id,
                team_position: p.team_position.clone(),
                kills: p.kills,
                deaths: p.deaths,
                assists: p.assists,
                total_minions_killed: p.total_minions_killed,
                gold_earned: p.gold_earned,
                total_damage_dealt: p.total_damage_dealt,
                vision_score: p.vision_score,
                win: p.win,
            })
            .collect();

        // Store match
        self.db.store_match(
            match_id,
            riot_match.info.game_creation,
            riot_match.info.game_duration,
            &riot_match.info.game_mode,
            riot_match.info.queue_id,
            Some(&riot_match.info.game_version),
            &raw_json,
            &participants,
        )?;

        // Fetch and store timeline
        match self.api.get_timeline(match_id).await {
            Ok((_timeline, raw_tl)) => {
                self.db.store_timeline(match_id, &raw_tl)?;
            }
            Err(e) => {
                tracing::warn!("Failed to fetch timeline for {match_id}: {e}");
                // Non-fatal: match data is still valuable without timeline
            }
        }

        tracing::info!("Stored match {match_id}");
        Ok(())
    }

    /// Background backfill: fetch and store recent matches at low priority
    pub async fn backfill(&self, puuid: &str, max_matches: i32) -> Result<i32> {
        let new_ids = self.get_new_match_ids(puuid, max_matches).await?;
        let total = new_ids.len() as i32;

        for (i, match_id) in new_ids.iter().enumerate() {
            // Emit progress to frontend
            let _ = self.app_handle.emit(
                "backfill-progress",
                serde_json::json!({
                    "current": i + 1,
                    "total": total,
                    "match_id": match_id,
                }),
            );

            if let Err(e) = self.fetch_and_store_match(match_id).await {
                tracing::warn!("Failed to fetch match {match_id}: {e}");
                // Continue with next match
            }

            // Small delay between fetches to be gentle on the API
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }

        let _ = self.app_handle.emit(
            "backfill-complete",
            serde_json::json!({ "fetched": total }),
        );

        Ok(total)
    }

    /// Fetch Data Dragon champion + item data and store locally
    pub async fn update_static_data(&self) -> Result<()> {
        let current_patch = self.db.get_state("ddragon_version")?;
        let latest_version = self.api.get_latest_version().await?;

        if current_patch.as_deref() == Some(&latest_version) {
            tracing::debug!("Static data already up to date ({})", latest_version);
            return Ok(());
        }

        tracing::info!("Updating static data to patch {latest_version}");

        // Fetch champions
        let champ_data = self.api.get_champions(&latest_version).await?;
        if let Some(data) = champ_data.get("data").and_then(|d| d.as_object()) {
            let conn = &self.db;
            let mut count = 0;
            for c in data.values() {
                let Some(key) = c.get("key").and_then(|v| v.as_str()).and_then(|v| v.parse::<i64>().ok()) else { continue };
                let Some(id_str) = c.get("id").and_then(|v| v.as_str()) else { continue };
                let Some(name) = c.get("name").and_then(|v| v.as_str()) else { continue };
                let title = c.get("title").and_then(|t| t.as_str());
                let tags = c.get("tags").map(|t| t.to_string());
                conn.store_champions(&[(key, id_str, name, title, tags.as_deref(), &latest_version)])?;
                count += 1;
            }
            tracing::info!("Stored {count} champions");
        }

        // Fetch items
        let item_data = self.api.get_items(&latest_version).await?;
        if let Some(data) = item_data.get("data").and_then(|d| d.as_object()) {
            let conn = &self.db;
            let mut count = 0;
            for (id_str, item) in data {
                let Some(id) = id_str.parse::<i64>().ok() else { continue };
                let Some(name) = item.get("name").and_then(|v| v.as_str()) else { continue };
                let desc = item.get("plaintext").and_then(|d| d.as_str());
                let gold = item.get("gold").and_then(|g| g.as_object());
                let gold_total = gold.and_then(|g| g.get("total")).and_then(|v| v.as_i64());
                let gold_base = gold.and_then(|g| g.get("base")).and_then(|v| v.as_i64());
                let tags = item.get("tags").map(|t| t.to_string());
                let from = item.get("from").map(|f| f.to_string());
                let into_items = item.get("into").map(|i| i.to_string());
                conn.store_items(&[(id, name, desc, gold_total, gold_base, tags.as_deref(), from.as_deref(), into_items.as_deref(), &latest_version)])?;
                count += 1;
            }
            tracing::info!("Stored {count} items");
        }

        self.db.set_state("ddragon_version", &latest_version)?;
        Ok(())
    }
}
