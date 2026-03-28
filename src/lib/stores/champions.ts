import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

/** Map of champion_id -> champion name (from Data Dragon) */
export const championMap = writable<Record<number, string>>({});

/** Load champion map from backend */
export async function loadChampionMap() {
  try {
    const map = await invoke<Record<string, string>>("get_champion_map");
    // Keys come as strings from JSON, convert to number keys
    const numMap: Record<number, string> = {};
    for (const [id, name] of Object.entries(map)) {
      numMap[Number(id)] = name;
    }
    championMap.set(numMap);
  } catch (e) {
    console.error("Failed to load champion map:", e);
  }
}

/** Get champion name by ID, fallback to "Champion {id}" */
export function getChampionName(map: Record<number, string>, id: number): string {
  if (id <= 0) return "";
  return map[id] ?? `Champion ${id}`;
}

// Load on init
loadChampionMap();
