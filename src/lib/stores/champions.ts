import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface ChampionInfo {
  name: string;
  key: string; // Data Dragon key (e.g., "MissFortune")
  patch: string;
}

/** Map of champion_id -> ChampionInfo */
export const championMap = writable<Record<number, ChampionInfo>>({});

/** Current patch version for image URLs */
export const currentPatch = writable<string>("15.6.1");

/** Load champion map from backend */
export async function loadChampionMap() {
  try {
    const raw = await invoke<Record<string, { name: string; key: string; patch: string }>>(
      "get_champion_map"
    );
    const map: Record<number, ChampionInfo> = {};
    let patch = "15.6.1";
    for (const [id, info] of Object.entries(raw)) {
      map[Number(id)] = info;
      patch = info.patch; // All same patch
    }
    championMap.set(map);
    currentPatch.set(patch);
  } catch (e) {
    console.error("Failed to load champion map:", e);
  }
}

/** Get champion name by ID */
export function getChampionName(map: Record<number, ChampionInfo>, id: number): string {
  if (id <= 0) return "";
  return map[id]?.name ?? `Champion ${id}`;
}

/** Get champion square image URL from Data Dragon */
export function getChampionImageUrl(map: Record<number, ChampionInfo>, id: number, patch: string): string {
  if (id <= 0) return "";
  const key = map[id]?.key;
  if (!key) return "";
  return `https://ddragon.leagueoflegends.com/cdn/${patch}/img/champion/${key}.png`;
}

// Load on init
loadChampionMap();
