import { writable, derived } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import type { LiveGameState, PowerSpike } from "../types/livegame.js";

export const liveGameState = writable<LiveGameState | null>(null);
export const powerSpikeAlerts = writable<PowerSpike[]>([]);

// Listen for live game updates from Rust backend (1Hz)
listen<LiveGameState>("live-game-update", (event) => {
  liveGameState.set(event.payload);
});

// Listen for power spike alerts
listen<PowerSpike>("power-spike", (event) => {
  powerSpikeAlerts.update((alerts) => {
    const updated = [...alerts, event.payload];
    // Keep last 5 alerts
    return updated.slice(-5);
  });
  // Auto-dismiss after 8 seconds
  setTimeout(() => {
    powerSpikeAlerts.update((alerts) => alerts.slice(1));
  }, 8000);
});

// Derived: is a game active
export const isGameActive = derived(liveGameState, ($state) => $state !== null && $state.game_time > 0);
