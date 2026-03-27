import { writable } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type { ConnectionStatus, Summoner, GameFlowPhase } from "../types/index.js";
import { gamePhase } from "./gameflow.js";

export const connectionStatus = writable<ConnectionStatus>("disconnected");
export const currentSummoner = writable<Summoner | null>(null);

// Listen for LCU events from Rust backend
listen<LcuEvent>("lcu-event", (event) => {
  const payload = event.payload;

  switch (payload.type) {
    case "Connected":
      connectionStatus.set("connected");
      currentSummoner.set(payload.summoner ?? null);
      break;
    case "Disconnected":
      connectionStatus.set("disconnected");
      currentSummoner.set(null);
      gamePhase.set("None");
      break;
    case "GameFlowChanged":
      gamePhase.set((payload.phase as GameFlowPhase) ?? "None");
      break;
  }
});

// Listen for status updates (connecting state)
listen<{ status: string; game_phase: string }>("lcu-status", (event) => {
  const { status, game_phase } = event.payload;
  if (status === "connecting") {
    connectionStatus.set("connecting");
  }
  if (game_phase) {
    gamePhase.set(game_phase as GameFlowPhase);
  }
});

// Fetch initial state on load
invoke<{ status: string; summoner: Summoner | null; game_phase: string }>(
  "get_connection_status"
).then((state) => {
  connectionStatus.set(state.status as ConnectionStatus);
  currentSummoner.set(state.summoner);
  gamePhase.set((state.game_phase as GameFlowPhase) ?? "None");
});

// Type for LCU events from the backend
interface LcuEventConnected {
  type: "Connected";
  summoner: Summoner;
}
interface LcuEventDisconnected {
  type: "Disconnected";
}
interface LcuEventGameFlow {
  type: "GameFlowChanged";
  phase: string;
}
interface LcuEventChampSelect {
  type: "ChampSelectUpdate";
  data: unknown;
}
interface LcuEventEndOfGame {
  type: "EndOfGame";
  data: unknown;
}

type LcuEvent =
  | LcuEventConnected
  | LcuEventDisconnected
  | LcuEventGameFlow
  | LcuEventChampSelect
  | LcuEventEndOfGame;
