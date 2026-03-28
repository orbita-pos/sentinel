use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::HashMap;

const OPGG_MCP_URL: &str = "https://mcp-api.op.gg/mcp";

/// Champion build data from OP.GG (millions of matches)
#[derive(Debug, Clone, Serialize, Default)]
pub struct ChampionBuild {
    pub champion: String,
    pub position: String,
    pub win_rate: f64,
    pub pick_rate: f64,
    pub ban_rate: f64,
    pub tier: String,
    pub core_items: BuildSet,
    pub boots: BuildSet,
    pub starter_items: BuildSet,
    pub situational_items: Vec<BuildSet>,
    pub runes: RuneSet,
    pub skill_order: Vec<String>,
    pub counters: Vec<CounterInfo>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct BuildSet {
    pub item_ids: Vec<i64>,
    pub item_names: Vec<String>,
    pub pick_rate: f64,
    pub games: i64,
    pub wins: i64,
    pub win_rate: f64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct RuneSet {
    pub primary_tree: String,
    pub primary_runes: Vec<String>,
    pub secondary_tree: String,
    pub secondary_runes: Vec<String>,
    pub stat_mods: Vec<String>,
    pub pick_rate: f64,
    pub win_rate: f64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct CounterInfo {
    pub champion: String,
    pub win_rate: f64,
    pub games: i64,
}

/// Convert champion name to OP.GG format (UPPER_SNAKE_CASE)
fn to_opgg_name(name: &str) -> String {
    // Handle special cases
    let mut result = String::new();
    let chars: Vec<char> = name.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && c.is_uppercase() && !chars[i - 1].is_uppercase() {
            result.push('_');
        }
        result.push(c.to_ascii_uppercase());
    }

    // Fix known special cases
    result
        .replace("JARVAN_I_V", "JARVAN_IV")
        .replace("REK_SAI", "REKSAI")
        .replace("KOG_MAW", "KOGMAW")
        .replace("VEL_KOZ", "VELKOZ")
        .replace("K_SANTE", "KSANTE")
        .replace("BEL_VETH", "BELVETH")
        .replace("TAH_M_KENCH", "TAHM_KENCH")
        .replace("MISS_FORTUNE", "MISS_FORTUNE")
        .replace("LEE_SIN", "LEE_SIN")
        .replace("TWISTED_FATE", "TWISTED_FATE")
        .replace("DR_MUNDO", "DR_MUNDO")
        .replace("MASTER_YI", "MASTER_YI")
        .replace("XIN_ZHAO", "XIN_ZHAO")
        .replace("AURELION_SOL", "AURELION_SOL")
}

/// Fetch champion build data from OP.GG MCP server
pub async fn fetch_champion_build(champion: &str, position: &str) -> Result<ChampionBuild> {
    let opgg_name = to_opgg_name(champion);
    let pos = match position.to_lowercase().as_str() {
        "top" => "top",
        "jungle" | "jng" => "jungle",
        "mid" | "middle" => "mid",
        "bot" | "bottom" | "adc" => "adc",
        "support" | "sup" | "utility" => "support",
        _ => "all",
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "lol_get_champion_analysis",
            "arguments": {
                "game_mode": "ranked",
                "champion": opgg_name,
                "position": pos,
                "desired_output_fields": [
                    "data.core_items.{ids[],ids_names[],pick_rate,play,win}",
                    "data.boots.{ids[],ids_names[],pick_rate,play,win}",
                    "data.starter_items.{ids[],ids_names[],pick_rate,play,win}",
                    "data.fourth_items[].{ids[],ids_names[],pick_rate,play,win}",
                    "data.fifth_items[].{ids[],ids_names[],pick_rate,play,win}",
                    "data.sixth_items[].{ids[],ids_names[],pick_rate,play,win}",
                    "data.runes.{primary_page_name,primary_rune_names[],secondary_page_name,secondary_rune_names[],stat_mod_names[],pick_rate,win}",
                    "data.skills.{order[],pick_rate,play,win}",
                    "data.summary.average_stats.{win_rate,pick_rate,ban_rate,kda,tier}",
                    "data.strong_counters[].{champion_name,win_rate,play}"
                ]
            }
        }
    });

    let resp = client
        .post(OPGG_MCP_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&body)
        .send()
        .await
        .context("OP.GG MCP request failed")?;

    let status = resp.status();
    let resp_text = resp.text().await.context("Failed to read OP.GG response")?;

    if !status.is_success() {
        anyhow::bail!("OP.GG returned {status}");
    }

    // Parse JSON-RPC response
    let rpc: serde_json::Value = serde_json::from_str(&resp_text)
        .context("Failed to parse OP.GG JSON-RPC response")?;

    let content_text = rpc
        .get("result")
        .and_then(|r| r.get("content"))
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("");

    // Parse the custom class-notation format
    parse_opgg_response(content_text, champion, pos)
}

/// Parse OP.GG's custom class-notation response into structured data
fn parse_opgg_response(text: &str, champion: &str, position: &str) -> Result<ChampionBuild> {
    let mut build = ChampionBuild {
        champion: champion.to_string(),
        position: position.to_string(),
        ..Default::default()
    };

    // The response has class definitions at the top, then data
    // Format: ClassName(field1, field2, ...)
    // We'll extract data using regex-like pattern matching

    // Extract core items: CoreItems([ids],[names],pick_rate,play,win)
    if let Some(core) = extract_build_set(text, "CoreItems") {
        // First CoreItems is the build, second might be boots
        let all_cores = extract_all_build_sets(text, "CoreItems");
        if all_cores.len() >= 2 {
            build.core_items = all_cores[0].clone();
            build.boots = all_cores[1].clone();
        } else if !all_cores.is_empty() {
            build.core_items = all_cores[0].clone();
        }
    }

    // Extract starter items
    if let Some(starter) = extract_build_set(text, "StarterItems") {
        build.starter_items = starter;
    }

    // Extract situational items (4th, 5th, 6th)
    let fourth = extract_all_build_sets(text, "FourthItems");
    let fifth = extract_all_build_sets(text, "FifthItems");
    let sixth = extract_all_build_sets(text, "SixthItems");
    build.situational_items = [fourth, fifth, sixth].concat();

    // Extract runes
    build.runes = extract_runes(text);

    // Extract summary stats
    if let Some(stats) = extract_summary(text) {
        build.win_rate = stats.0;
        build.pick_rate = stats.1;
        build.ban_rate = stats.2;
        build.tier = stats.3;
    }

    // Extract skill order
    build.skill_order = extract_skill_order(text);

    // Extract counters
    build.counters = extract_counters(text);

    // Calculate win rates for build sets
    if build.core_items.games > 0 {
        build.core_items.win_rate = build.core_items.wins as f64 / build.core_items.games as f64;
    }
    if build.boots.games > 0 {
        build.boots.win_rate = build.boots.wins as f64 / build.boots.games as f64;
    }

    Ok(build)
}

fn extract_all_build_sets(text: &str, class_name: &str) -> Vec<BuildSet> {
    let mut results = Vec::new();
    let pattern = format!("{class_name}(");
    let mut search_from = 0;

    while let Some(start) = text[search_from..].find(&pattern) {
        let abs_start = search_from + start + pattern.len();
        if let Some(end) = find_matching_paren(text, abs_start - 1) {
            let inner = &text[abs_start..end];
            if let Some(bs) = parse_build_set_inner(inner) {
                results.push(bs);
            }
            search_from = end + 1;
        } else {
            break;
        }
    }
    results
}

fn extract_build_set(text: &str, class_name: &str) -> Option<BuildSet> {
    let sets = extract_all_build_sets(text, class_name);
    sets.into_iter().next()
}

fn parse_build_set_inner(inner: &str) -> Option<BuildSet> {
    // Format: [id1,id2,...],[name1,name2,...],pick_rate,play,win
    let mut bs = BuildSet::default();

    // Extract first array [ids]
    if let (Some(ids_start), Some(ids_end)) = (inner.find('['), inner.find(']')) {
        let ids_str = &inner[ids_start + 1..ids_end];
        bs.item_ids = ids_str
            .split(',')
            .filter_map(|s| s.trim().parse::<i64>().ok())
            .collect();

        let rest = &inner[ids_end + 1..];
        // Extract second array [names]
        if let (Some(names_start), Some(names_end)) = (rest.find('['), rest.find(']')) {
            let names_str = &rest[names_start + 1..names_end];
            bs.item_names = names_str
                .split("\",\"")
                .map(|s| s.trim().trim_matches('"').to_string())
                .filter(|s| !s.is_empty())
                .collect();

            // Parse remaining numbers: pick_rate, play, win
            let nums_str = &rest[names_end + 1..];
            let nums: Vec<f64> = nums_str
                .split(',')
                .filter_map(|s| s.trim().parse::<f64>().ok())
                .collect();

            if nums.len() >= 3 {
                bs.pick_rate = nums[0];
                bs.games = nums[1] as i64;
                bs.wins = nums[2] as i64;
                if bs.games > 0 {
                    bs.win_rate = bs.wins as f64 / bs.games as f64;
                }
            }
        }
    }

    if bs.item_ids.is_empty() {
        None
    } else {
        Some(bs)
    }
}

fn extract_runes(text: &str) -> RuneSet {
    let mut runes = RuneSet::default();
    // Look for Runes( pattern
    if let Some(start) = text.find("Runes(") {
        let inner_start = start + 6;
        if let Some(end) = find_matching_paren(text, start + 5) {
            let inner = &text[inner_start..end];
            // Extract quoted strings and numbers
            let quoted: Vec<String> = extract_quoted_strings(inner);
            let nums: Vec<f64> = inner
                .split(',')
                .filter_map(|s| {
                    let t = s.trim().trim_matches('"').trim_matches('[').trim_matches(']');
                    t.parse::<f64>().ok()
                })
                .collect();

            if !quoted.is_empty() {
                runes.primary_tree = quoted.first().cloned().unwrap_or_default();
                // Rune names are in arrays, hard to parse exactly
                // Just capture what we can
                if quoted.len() > 1 {
                    runes.secondary_tree = quoted.last().cloned().unwrap_or_default();
                }
                runes.primary_runes = quoted.get(1..5).unwrap_or_default().to_vec();
            }
            if nums.len() >= 2 {
                runes.pick_rate = *nums.last().unwrap_or(&0.0);
                runes.win_rate = nums.get(nums.len().wrapping_sub(1)).copied().unwrap_or(0.0);
            }
        }
    }
    runes
}

fn extract_summary(text: &str) -> Option<(f64, f64, f64, String)> {
    // Look for AverageStats(win_rate, pick_rate, ban_rate, kda, tier)
    if let Some(start) = text.find("AverageStats(") {
        let inner_start = start + 13;
        if let Some(end) = find_matching_paren(text, start + 12) {
            let inner = &text[inner_start..end];
            let parts: Vec<&str> = inner.split(',').collect();
            let nums: Vec<f64> = parts.iter().filter_map(|s| s.trim().trim_matches('"').parse().ok()).collect();
            let tier = parts.last().map(|s| s.trim().trim_matches('"').to_string()).unwrap_or_default();

            if nums.len() >= 3 {
                return Some((nums[0], nums[1], nums[2], tier));
            }
        }
    }
    None
}

fn extract_skill_order(text: &str) -> Vec<String> {
    if let Some(start) = text.find("Skills(") {
        let inner_start = start + 7;
        if let Some(end) = find_matching_paren(text, start + 6) {
            let inner = &text[inner_start..end];
            if let (Some(arr_start), Some(arr_end)) = (inner.find('['), inner.find(']')) {
                let arr = &inner[arr_start + 1..arr_end];
                return arr.split(',')
                    .map(|s| s.trim().trim_matches('"').to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }
    vec![]
}

fn extract_counters(text: &str) -> Vec<CounterInfo> {
    let mut counters = Vec::new();
    let pattern = "StrongCounters(";
    let mut pos = 0;
    while let Some(start) = text[pos..].find(pattern) {
        let abs = pos + start + pattern.len();
        if let Some(end) = find_matching_paren(text, abs - 1) {
            let inner = &text[abs..end];
            let quoted = extract_quoted_strings(inner);
            let nums: Vec<f64> = inner.split(',')
                .filter_map(|s| s.trim().trim_matches('"').parse().ok())
                .collect();

            if let Some(name) = quoted.first() {
                counters.push(CounterInfo {
                    champion: name.clone(),
                    win_rate: nums.first().copied().unwrap_or(0.0),
                    games: nums.get(1).copied().unwrap_or(0.0) as i64,
                });
            }
            pos = end + 1;
        } else {
            break;
        }
    }
    counters
}

fn find_matching_paren(text: &str, open_pos: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    if open_pos >= bytes.len() || bytes[open_pos] != b'(' { return None; }
    let mut depth = 1;
    for i in (open_pos + 1)..bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 { return Some(i); }
            }
            _ => {}
        }
    }
    None
}

fn extract_quoted_strings(text: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut in_quote = false;
    let mut current = String::new();

    for c in text.chars() {
        if c == '"' {
            if in_quote {
                if !current.is_empty() {
                    results.push(current.clone());
                }
                current.clear();
            }
            in_quote = !in_quote;
        } else if in_quote {
            current.push(c);
        }
    }
    results
}

/// Meta champion entry for tier lists
#[derive(Debug, Clone, Serialize, Default)]
pub struct MetaChampion {
    pub champion: String,
    pub tier: String,
    pub win_rate: f64,
    pub pick_rate: f64,
    pub ban_rate: f64,
    pub position: String,
}

/// Fetch meta tier list for a lane from OP.GG
pub async fn fetch_meta_tierlist(position: &str) -> Result<Vec<MetaChampion>> {
    let pos = match position.to_lowercase().as_str() {
        "top" => "top", "jungle" | "jng" => "jungle",
        "mid" | "middle" => "mid", "bot" | "bottom" | "adc" => "adc",
        "support" | "sup" | "utility" => "support", _ => "all",
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "lol_list_lane_meta_champions",
            "arguments": {
                "game_mode": "ranked",
                "position": pos
            }
        }
    });

    let resp = client
        .post(OPGG_MCP_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&body)
        .send()
        .await
        .context("OP.GG meta tierlist request failed")?;

    let resp_text = resp.text().await?;
    let rpc: serde_json::Value = serde_json::from_str(&resp_text)?;

    let content_text = rpc
        .get("result")
        .and_then(|r| r.get("content"))
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("");

    parse_meta_tierlist(content_text, pos)
}

fn parse_meta_tierlist(text: &str, position: &str) -> Result<Vec<MetaChampion>> {
    let mut champions = Vec::new();

    // Look for champion entries in the response
    // Format varies but typically: ChampionName, tier, winrate, pickrate, banrate
    let quoted = extract_quoted_strings(text);
    let nums: Vec<f64> = text.split(',')
        .filter_map(|s| s.trim().trim_matches('"').trim_matches(')').trim_matches('(').parse().ok())
        .collect();

    // Parse in groups -- each champion has a name + numbers
    // Simple heuristic: quoted strings are champion names, numbers follow
    let mut num_idx = 0;
    for name in &quoted {
        // Skip non-champion strings (tier labels, etc.)
        if name.len() < 2 || name.contains(' ') || name.starts_with('[') {
            continue;
        }

        let win_rate = nums.get(num_idx).copied().unwrap_or(0.0);
        let pick_rate = nums.get(num_idx + 1).copied().unwrap_or(0.0);
        let ban_rate = nums.get(num_idx + 2).copied().unwrap_or(0.0);

        // Determine tier from win rate
        let tier = if win_rate >= 0.53 { "S" }
            else if win_rate >= 0.51 { "A" }
            else if win_rate >= 0.49 { "B" }
            else { "C" };

        if win_rate > 0.0 && win_rate < 1.0 {
            champions.push(MetaChampion {
                champion: name.clone(),
                tier: tier.to_string(),
                win_rate,
                pick_rate,
                ban_rate,
                position: position.to_string(),
            });
            num_idx += 3;
        }
    }

    // If parsing failed, just return what we have
    Ok(champions)
}

/// Cache for champion builds (avoid re-fetching during the same game)
pub struct BuildCache {
    cache: HashMap<String, (std::time::Instant, ChampionBuild)>,
}

impl BuildCache {
    pub fn new() -> Self {
        Self { cache: HashMap::new() }
    }

    /// Get cached build or fetch from OP.GG (cache for 10 minutes)
    pub async fn get(&mut self, champion: &str, position: &str) -> Option<ChampionBuild> {
        let key = format!("{champion}_{position}");

        if let Some((time, build)) = self.cache.get(&key) {
            if time.elapsed().as_secs() < 600 {
                return Some(build.clone());
            }
        }

        match fetch_champion_build(champion, position).await {
            Ok(build) => {
                self.cache.insert(key, (std::time::Instant::now(), build.clone()));
                Some(build)
            }
            Err(e) => {
                tracing::warn!("Failed to fetch OP.GG build for {champion}: {e}");
                None
            }
        }
    }
}
