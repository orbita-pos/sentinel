import { writable } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import type { ChampSelectSession } from "../types/champselect.js";

export const champSelectSession = writable<ChampSelectSession | null>(null);

// Listen for parsed champ select updates from backend
listen<ChampSelectSession>("champ-select-update", (event) => {
  champSelectSession.set(event.payload);
});
