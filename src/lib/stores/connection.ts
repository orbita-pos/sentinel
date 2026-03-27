import { writable } from "svelte/store";
import type { ConnectionStatus, Summoner } from "../types/index.js";

export const connectionStatus = writable<ConnectionStatus>("disconnected");
export const currentSummoner = writable<Summoner | null>(null);
