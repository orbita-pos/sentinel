export interface LiveGameState {
  game_time: number;
  game_mode: string;
  my_team: LivePlayerState[];
  enemy_team: LivePlayerState[];
  team_gold_diff: number;
  gold_diff_history: GoldDiffPoint[];
  recent_events: LiveEvent[];
  power_spikes: PowerSpike[];
  objective_events: LiveEvent[];
  active_player: ActivePlayerState;
}

export interface ActivePlayerState {
  champion: string;
  level: number;
  current_gold: number;
  runes: any;
  champion_stats: any;
}

export interface LivePlayerState {
  name: string;
  champion: string;
  team: string;
  level: number;
  kills: number;
  deaths: number;
  assists: number;
  cs: number;
  ward_score: number;
  items: LiveItem[];
  is_local_player: boolean;
}

export interface LiveItem {
  item_id: number;
  name: string;
  price: number;
}

export interface GoldDiffPoint {
  time: number;
  diff: number;
}

export interface PowerSpike {
  player_name: string;
  champion: string;
  team: string;
  spike_type: string;
  description: string;
  game_time: number;
}

export interface LiveEvent {
  event_name: string;
  event_time: number;
  description: string;
}
