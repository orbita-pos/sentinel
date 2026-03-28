<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionStatus, currentSummoner } from "../lib/stores/connection.js";
  import type { DbStats } from "../lib/types/index.js";

  let dbStats: DbStats | null = $state(null);
  let appVersion: string = $state("");

  $effect(() => {
    invoke<string>("get_app_version").then((v) => (appVersion = v));
    invoke<DbStats>("get_db_stats").then((s) => (dbStats = s));
  });

  let status = $derived($connectionStatus);
  let summoner = $derived($currentSummoner);
</script>

<div class="mx-auto max-w-4xl">
  <!-- Header -->
  <div class="mb-8">
    <h2 class="text-2xl font-bold" style="color: var(--text-primary)">Dashboard</h2>
    <p class="mt-1 text-sm" style="color: var(--text-secondary)">Personal gameplay intelligence for League of Legends</p>
  </div>

  <!-- Status Cards -->
  <div class="grid grid-cols-3 gap-4 mb-8">
    <!-- Connection Status -->
    <div class="rounded-xl border p-5" style="background: var(--bg-secondary); border-color: var(--border)">
      <p class="text-xs font-medium uppercase tracking-wide" style="color: var(--text-muted)">League Client</p>
      <p class="mt-2 text-lg font-semibold" style="color: {status === 'connected' ? 'var(--accent-green)' : 'var(--text-secondary)'}">
        {status === "connected" ? "Connected" : status === "connecting" ? "Connecting..." : "Not Detected"}
      </p>
      {#if summoner}
        <p class="mt-1 text-sm" style="color: var(--text-secondary)">{summoner.game_name}#{summoner.tag_line}</p>
      {:else}
        <p class="mt-1 text-sm" style="color: var(--text-muted)">Launch League to connect</p>
      {/if}
    </div>

    <!-- Games Analyzed -->
    <div class="rounded-xl border p-5" style="background: var(--bg-secondary); border-color: var(--border)">
      <p class="text-xs font-medium uppercase tracking-wide" style="color: var(--text-muted)">Games Analyzed</p>
      <p class="mt-2 text-lg font-semibold" style="color: var(--text-primary)">0</p>
      <p class="mt-1 text-sm" style="color: var(--text-muted)">Connect and play to start</p>
    </div>

    <!-- Patterns Detected -->
    <div class="rounded-xl border p-5" style="background: var(--bg-secondary); border-color: var(--border)">
      <p class="text-xs font-medium uppercase tracking-wide" style="color: var(--text-muted)">Patterns Detected</p>
      <p class="mt-2 text-lg font-semibold" style="color: var(--text-primary)">--</p>
      <p class="mt-1 text-sm" style="color: var(--text-muted)">Need 10+ games</p>
    </div>
  </div>

  <!-- Info Section -->
  <div class="rounded-xl border p-6" style="background: var(--bg-secondary); border-color: var(--border)">
    <h3 class="text-sm font-semibold" style="color: var(--text-primary)">Getting Started</h3>
    <ul class="mt-3 space-y-2 text-sm" style="color: var(--text-secondary)">
      <li class="flex items-center gap-2">
        <span class="h-1.5 w-1.5 rounded-full" style="background: {status === 'connected' ? 'var(--accent-green)' : 'var(--accent-red)'}"></span>
        {status === "connected" ? "League client detected -- match data syncing" : "Launch League of Legends client"}
      </li>
      <li class="flex items-center gap-2">
        <span class="h-1.5 w-1.5 rounded-full" style="background: var(--text-muted)"></span>
        Play games to build your personal data
      </li>
      <li class="flex items-center gap-2">
        <span class="h-1.5 w-1.5 rounded-full" style="background: var(--text-muted)"></span>
        View patterns and improvement after 10+ games
      </li>
    </ul>
    <p class="mt-3 text-xs" style="color: var(--text-muted)">
      Want deeper analysis (CS timing, gold leads, death patterns)?
      Add an optional API key in Settings.
    </p>
  </div>

  <!-- Debug Info -->
  {#if dbStats}
    <div class="mt-6 rounded-lg border p-4" style="background: var(--bg-tertiary); border-color: var(--border)">
      <p class="text-xs font-mono" style="color: var(--text-muted)">
        Sentinel v{appVersion} | DB: {dbStats.db_path}
      </p>
    </div>
  {/if}
</div>
