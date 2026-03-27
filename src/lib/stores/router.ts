import { writable } from "svelte/store";
import type { Route } from "../types/index.js";

export const currentRoute = writable<Route>("dashboard");
