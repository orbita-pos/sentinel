export type ConnectionStatus = "disconnected" | "connecting" | "connected";

export type GameFlowPhase =
  | "None"
  | "Lobby"
  | "Matchmaking"
  | "ReadyCheck"
  | "ChampSelect"
  | "GameStart"
  | "InProgress"
  | "WaitingForStats"
  | "PreEndOfGame"
  | "EndOfGame"
  | "Reconnect";

export interface Summoner {
  puuid: string;
  game_name: string;
  tag_line: string;
  summoner_id?: string;
  profile_icon_id?: number;
  summoner_level?: number;
  region: string;
}

export interface DbStats {
  db_path: string;
  summoners: number;
  app_state_entries: number;
}

export interface MatchSummaryItem {
  match_id: string;
  game_creation: number;
  game_duration: number;
  game_mode: string;
  queue_id: number;
  champion_name: string;
  champion_id: number;
  kills: number;
  deaths: number;
  assists: number;
  cs: number;
  gold: number;
  vision_score: number;
  win: boolean;
  role: string | null;
}

export type Route =
  | "dashboard"
  | "match-history"
  | "champ-select"
  | "live-game"
  | "post-game"
  | "patterns"
  | "improvement"
  | "settings";
