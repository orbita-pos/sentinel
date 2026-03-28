use serde::Serialize;

use super::types::{AllPlayer, EventData, GameStats};

/// Composite live game state pushed to the frontend every tick
#[derive(Debug, Clone, Serialize, Default)]
pub struct LiveGameState {
    pub game_time: f64,
    pub game_mode: String,
    pub my_team: Vec<LivePlayerState>,
    pub enemy_team: Vec<LivePlayerState>,
    pub team_gold_diff: i64,
    pub gold_diff_history: Vec<GoldDiffPoint>,
    pub recent_events: Vec<LiveEvent>,
    pub power_spikes: Vec<PowerSpike>,
    pub objective_events: Vec<LiveEvent>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct LivePlayerState {
    pub name: String,
    pub champion: String,
    pub team: String,
    pub level: i64,
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
    pub cs: i64,
    pub ward_score: f64,
    pub items: Vec<LiveItem>,
    pub is_local_player: bool,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct LiveItem {
    pub item_id: i64,
    pub name: String,
    pub price: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct GoldDiffPoint {
    pub time: f64,
    pub diff: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PowerSpike {
    pub player_name: String,
    pub champion: String,
    pub team: String,
    pub spike_type: String,
    pub description: String,
    pub game_time: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LiveEvent {
    pub event_name: String,
    pub event_time: f64,
    pub description: String,
}

/// Major items that constitute a power spike (common completed items by gold cost)
const MAJOR_ITEM_MIN_PRICE: i64 = 2600;

impl LiveGameState {
    /// Update state from polled API data, detecting changes
    pub fn update(
        &mut self,
        players: &[AllPlayer],
        events: &EventData,
        stats: &GameStats,
        local_player_name: &str,
    ) {
        let prev_items = self.collect_item_sets();

        self.game_time = stats.game_time;
        self.game_mode = stats.game_mode.clone();

        // Determine local player's team
        let local_team = players
            .iter()
            .find(|p| p.riot_id_game_name == local_player_name)
            .map(|p| p.team.clone())
            .unwrap_or_else(|| "ORDER".to_string());

        // Split players into teams
        let mut my_team = Vec::new();
        let mut enemy_team = Vec::new();
        let mut my_gold: i64 = 0;
        let mut enemy_gold: i64 = 0;

        for p in players {
            let player = LivePlayerState {
                name: p.riot_id_game_name.clone(),
                champion: p.champion_name.clone(),
                team: p.team.clone(),
                level: p.level,
                kills: p.scores.kills,
                deaths: p.scores.deaths,
                assists: p.scores.assists,
                cs: p.scores.creep_score,
                ward_score: p.scores.ward_score,
                items: p
                    .items
                    .iter()
                    .filter(|i| i.item_id > 0)
                    .map(|i| LiveItem {
                        item_id: i.item_id,
                        name: i.display_name.clone(),
                        price: i.price,
                    })
                    .collect(),
                is_local_player: p.riot_id_game_name == local_player_name,
            };

            // Estimate gold from items + current visible gold
            let item_gold: i64 = p.items.iter().map(|i| i.price * i.count).sum();

            if p.team == local_team {
                my_gold += item_gold;
                my_team.push(player);
            } else {
                enemy_gold += item_gold;
                enemy_team.push(player);
            }
        }

        self.my_team = my_team;
        self.enemy_team = enemy_team;
        self.team_gold_diff = my_gold - enemy_gold;

        // Record gold diff point (every ~5 seconds to keep history manageable)
        let should_record = self
            .gold_diff_history
            .last()
            .map(|p| stats.game_time - p.time >= 5.0)
            .unwrap_or(true);
        if should_record {
            self.gold_diff_history.push(GoldDiffPoint {
                time: stats.game_time,
                diff: self.team_gold_diff,
            });
            // Cap history at 360 points (~30 minutes at 5s intervals)
            if self.gold_diff_history.len() > 360 {
                self.gold_diff_history.remove(0);
            }
        }

        // Detect power spikes from new items
        let new_items = self.collect_item_sets();
        self.detect_power_spikes(&prev_items, &new_items, stats.game_time);

        // Process game events (objectives)
        self.process_events(events);
    }

    fn collect_item_sets(&self) -> Vec<(String, Vec<i64>)> {
        self.my_team
            .iter()
            .chain(self.enemy_team.iter())
            .map(|p| {
                let ids: Vec<i64> = p.items.iter().map(|i| i.item_id).collect();
                (p.name.clone(), ids)
            })
            .collect()
    }

    fn detect_power_spikes(
        &mut self,
        prev: &[(String, Vec<i64>)],
        curr: &[(String, Vec<i64>)],
        game_time: f64,
    ) {
        for (name, curr_items) in curr {
            let prev_items = prev
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, items)| items.as_slice())
                .unwrap_or(&[]);

            // Find newly completed items
            for &item_id in curr_items {
                if !prev_items.contains(&item_id) {
                    // Check if this is a major item
                    let player = self
                        .my_team
                        .iter()
                        .chain(self.enemy_team.iter())
                        .find(|p| p.name == *name);

                    if let Some(player) = player {
                        let item = player.items.iter().find(|i| i.item_id == item_id);
                        if let Some(item) = item {
                            if item.price >= MAJOR_ITEM_MIN_PRICE {
                                self.power_spikes.push(PowerSpike {
                                    player_name: name.clone(),
                                    champion: player.champion.clone(),
                                    team: player.team.clone(),
                                    spike_type: "ItemCompleted".to_string(),
                                    description: format!(
                                        "{} ({}) completed {}",
                                        player.champion, name, item.name
                                    ),
                                    game_time,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Keep only recent power spikes (last 60 seconds)
        self.power_spikes
            .retain(|s| game_time - s.game_time < 60.0);
    }

    fn process_events(&mut self, events: &EventData) {
        let last_time = self
            .objective_events
            .last()
            .map(|e| e.event_time)
            .unwrap_or(0.0);

        for event in &events.events {
            if event.event_time <= last_time {
                continue;
            }

            let is_objective = matches!(
                event.event_name.as_str(),
                "DragonKill" | "BaronKill" | "HeraldKill" | "TurretKilled" | "InhibKilled"
            );

            if is_objective {
                self.objective_events.push(LiveEvent {
                    event_name: event.event_name.clone(),
                    event_time: event.event_time,
                    description: format_event(&event.event_name, event.event_time),
                });
            }

            // Keep recent events for display
            let is_notable = is_objective
                || matches!(
                    event.event_name.as_str(),
                    "ChampionKill" | "Multikill" | "Ace"
                );
            if is_notable && event.event_time > last_time {
                self.recent_events.push(LiveEvent {
                    event_name: event.event_name.clone(),
                    event_time: event.event_time,
                    description: format_event(&event.event_name, event.event_time),
                });
            }
        }

        // Keep last 20 recent events
        if self.recent_events.len() > 20 {
            let drain_count = self.recent_events.len() - 20;
            self.recent_events.drain(..drain_count);
        }
    }
}

fn format_event(name: &str, time: f64) -> String {
    let min = (time / 60.0) as i32;
    let sec = (time % 60.0) as i32;
    let label = match name {
        "DragonKill" => "Dragon slain",
        "BaronKill" => "Baron slain",
        "HeraldKill" => "Herald slain",
        "TurretKilled" => "Turret destroyed",
        "InhibKilled" => "Inhibitor destroyed",
        "ChampionKill" => "Kill",
        "Multikill" => "Multi-kill",
        "Ace" => "ACE",
        _ => name,
    };
    format!("{label} at {min}:{sec:02}")
}
