use std::sync::Arc;

use anyhow::{Context, Result};

use crate::database::Database;

/// Fetch and store Data Dragon static data (NO API key required -- public CDN)
pub async fn update_static_data(db: &Arc<Database>) -> Result<()> {
    let current_patch = db.get_state("ddragon_version")?;

    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    // Get latest version
    let versions: Vec<String> = http
        .get("https://ddragon.leagueoflegends.com/api/versions.json")
        .send()
        .await?
        .json()
        .await?;
    let latest_version = versions.first().context("No versions available")?.clone();

    if current_patch.as_deref() == Some(&latest_version) {
        tracing::debug!("Static data already up to date ({})", latest_version);
        return Ok(());
    }

    tracing::info!("Updating static data to patch {latest_version}");

    // Fetch champions
    let champ_url = format!(
        "https://ddragon.leagueoflegends.com/cdn/{latest_version}/data/en_US/champion.json"
    );
    let champ_data: serde_json::Value = http.get(&champ_url).send().await?.json().await?;

    if let Some(data) = champ_data.get("data").and_then(|d| d.as_object()) {
        let mut count = 0;
        for c in data.values() {
            let Some(key) = c.get("key").and_then(|v| v.as_str()).and_then(|v| v.parse::<i64>().ok()) else { continue };
            let Some(id_str) = c.get("id").and_then(|v| v.as_str()) else { continue };
            let Some(name) = c.get("name").and_then(|v| v.as_str()) else { continue };
            let title = c.get("title").and_then(|t| t.as_str());
            let tags = c.get("tags").map(|t| t.to_string());
            db.store_champions(&[(key, id_str, name, title, tags.as_deref(), &latest_version)])?;
            count += 1;
        }
        tracing::info!("Stored {count} champions");
    }

    // Fetch items
    let item_url = format!(
        "https://ddragon.leagueoflegends.com/cdn/{latest_version}/data/en_US/item.json"
    );
    let item_data: serde_json::Value = http.get(&item_url).send().await?.json().await?;

    if let Some(data) = item_data.get("data").and_then(|d| d.as_object()) {
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
            db.store_items(&[(id, name, desc, gold_total, gold_base, tags.as_deref(), from.as_deref(), into_items.as_deref(), &latest_version)])?;
            count += 1;
        }
        tracing::info!("Stored {count} items");
    }

    db.set_state("ddragon_version", &latest_version)?;
    Ok(())
}
