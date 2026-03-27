import { writable } from "svelte/store";
import type { GameFlowPhase } from "../types/index.js";

export const gamePhase = writable<GameFlowPhase>("None");
