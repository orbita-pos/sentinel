export interface ChampSelectSession {
  my_team: ChampSelectPlayer[];
  their_team: ChampSelectPlayer[];
  bans: number[];
  local_player_cell_id: number;
  phase: string;
  timer_remaining: number;
}

export interface ChampSelectPlayer {
  cell_id: number;
  champion_id: number;
  summoner_id: number;
  assigned_position: string;
  is_local_player: boolean;
  game_name?: string;
  tag_line?: string;
  puuid?: string;
  rank?: string;
}

export interface DraftRecommendation {
  champion_id: number;
  champion_name: string;
  score: number;
  personal_wr: number;
  personal_games: number;
  reasons: string[];
}

export interface ChampionPoolEntry {
  champion_id: number;
  champion_name: string;
  games: number;
  wins: number;
  win_rate: number;
  avg_kills: number;
  avg_deaths: number;
  avg_assists: number;
  avg_cs: number;
}
