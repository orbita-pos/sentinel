<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { connectionStatus, currentSummoner } from "../lib/stores/connection.js";
  import type { DbStats } from "../lib/types/index.js";

  let dbStats: DbStats | null = $state(null);
  let appVersion: string = $state("");
  let matchCount = $state(0);
  let hasKey = $state(false);

  // Import state
  let importing = $state(false);
  let importProgress = $state("");
  let importResult = $state("");

  $effect(() => {
    invoke<string>("get_app_version").then((v) => (appVersion = v));
    invoke<DbStats>("get_db_stats").then((s) => (dbStats = s));
    invoke<boolean>("has_api_key").then((v) => (hasKey = v));
  });

  // Load match count when summoner changes
  let status = $derived($connectionStatus);
  let summoner = $derived($currentSummoner);

  $effect(() => {
    if (summoner?.puuid) {
      invoke<{ total: number }>("get_match_history", { puuid: summoner.puuid, count: 1, offset: 0 })
        .then((r) => (matchCount = r.total));
    }
  });

  // Listen for import progress
  listen<{ phase: string; current?: number; total?: number; message: string }>("import-progress", (e) => {
    importProgress = e.payload.message;
    if (e.payload.current && e.payload.total) {
      importProgress = `${e.payload.current} / ${e.payload.total} matches`;
    }
  });
  listen<{ fetched: number }>("import-complete", (e) => {
    importProgress = "";
    importing = false;
    importResult = `Imported ${e.payload.fetched} matches`;
    // Refresh count
    if (summoner?.puuid) {
      invoke<{ total: number }>("get_match_history", { puuid: summoner.puuid, count: 1, offset: 0 })
        .then((r) => (matchCount = r.total));
    }
    setTimeout(() => (importResult = ""), 5000);
  });

  async function importHistory() {
    if (!summoner?.puuid) return;
    importing = true;
    importProgress = "Starting import...";
    importResult = "";
    try {
      const result = await invoke<{ matches_imported: number; features_extracted: number; patterns_detected: number }>(
        "import_full_history",
        { puuid: summoner.puuid, maxMatches: 300 }
      );
      importResult = `Imported ${result.matches_imported} matches, extracted ${result.features_extracted} features, detected ${result.patterns_detected} patterns`;
      matchCount = matchCount + result.matches_imported;
    } catch (e) {
      importResult = String(e);
    }
    importing = false;
    importProgress = "";
  }

  async function quickFetch() {
    if (!summoner?.puuid) return;
    importing = true;
    importProgress = "Fetching recent matches...";
    try {
      const result = await invoke<{ fetched: number }>("trigger_backfill", { puuid: summoner.puuid });
      importResult = `Fetched ${result.fetched} matches`;
      if (summoner?.puuid) {
        const r = await invoke<{ total: number }>("get_match_history", { puuid: summoner.puuid, count: 1, offset: 0 });
        matchCount = r.total;
      }
    } catch (e) {
      importResult = String(e);
    }
    importing = false;
    importProgress = "";
    setTimeout(() => (importResult = ""), 5000);
  }
</script>

<div class="mx-auto max-w-4xl">
  <div class="mb-8">
    <h2 class="text-2xl font-bold" style="color: var(--text-primary)">Dashboard</h2>
    <p class="mt-1 text-sm" style="color: var(--text-secondary)">Personal gameplay intelligence for League of Legends</p>
  </div>

  <!-- Status Cards -->
  <div class="grid grid-cols-3 gap-4 mb-6">
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

    <div class="rounded-xl border p-5" style="background: var(--bg-secondary); border-color: var(--border)">
      <p class="text-xs font-medium uppercase tracking-wide" style="color: var(--text-muted)">Games Analyzed</p>
      <p class="mt-2 text-lg font-semibold" style="color: var(--text-primary)">{matchCount}</p>
      <p class="mt-1 text-sm" style="color: var(--text-muted)">{matchCount === 0 ? "Import your history below" : "matches in database"}</p>
    </div>

    <div class="rounded-xl border p-5" style="background: var(--bg-secondary); border-color: var(--border)">
      <p class="text-xs font-medium uppercase tracking-wide" style="color: var(--text-muted)">Patterns Detected</p>
      <p class="mt-2 text-lg font-semibold" style="color: var(--text-primary)">--</p>
      <p class="mt-1 text-sm" style="color: var(--text-muted)">{matchCount >= 10 ? "Run analysis in Patterns" : "Need 10+ games"}</p>
    </div>
  </div>

  <!-- Import History Section -->
  {#if summoner?.puuid}
    <div class="mb-6 rounded-xl border p-6" style="background: var(--bg-secondary); border-color: var(--border)">
      <h3 class="text-sm font-semibold" style="color: var(--text-primary)">Import Match History</h3>
      <p class="mt-1 text-xs" style="color: var(--text-muted)">
        {matchCount === 0
          ? "Import your match history to start getting personalized insights"
          : `You have ${matchCount} matches. Import more for better analysis.`}
      </p>

      <div class="mt-4 flex gap-3">
        <!-- Quick fetch (no API key needed) -->
        <button
          onclick={quickFetch}
          class="rounded-lg px-4 py-2.5 text-sm font-medium text-white transition-colors disabled:opacity-50"
          style="background: var(--accent-blue)"
          disabled={importing}
        >
          Quick Fetch (50 recent)
        </button>

        <!-- Full import (needs API key) -->
        {#if hasKey}
          <button
            onclick={importHistory}
            class="rounded-lg px-4 py-2.5 text-sm font-medium text-white transition-colors disabled:opacity-50"
            style="background: var(--accent-purple)"
            disabled={importing}
          >
            Import Full History (300 games)
          </button>
        {:else}
          <div class="flex items-center rounded-lg border px-4 py-2.5" style="border-color: var(--border)">
            <span class="text-xs" style="color: var(--text-muted)">
              Add API key in Settings for full history (300+ games with timeline)
            </span>
          </div>
        {/if}
      </div>

      <!-- Progress -->
      {#if importing && importProgress}
        <div class="mt-3 flex items-center gap-3">
          <div class="h-1.5 flex-1 rounded-full" style="background: var(--bg-primary)">
            <div class="h-full animate-pulse rounded-full" style="background: var(--accent-blue); width: 100%"></div>
          </div>
          <span class="text-xs font-medium" style="color: var(--accent-blue)">{importProgress}</span>
        </div>
      {/if}

      <!-- Result -->
      {#if importResult}
        <p class="mt-3 text-xs" style="color: var(--accent-green)">{importResult}</p>
      {/if}
    </div>
  {/if}

  <!-- Getting Started -->
  <div class="rounded-xl border p-6" style="background: var(--bg-secondary); border-color: var(--border)">
    <h3 class="text-sm font-semibold" style="color: var(--text-primary)">How it works</h3>
    <ul class="mt-3 space-y-2 text-sm" style="color: var(--text-secondary)">
      <li class="flex items-center gap-2">
        <span class="h-1.5 w-1.5 rounded-full" style="background: {status === 'connected' ? 'var(--accent-green)' : 'var(--accent-red)'}"></span>
        {status === "connected" ? "League client detected" : "Launch League of Legends client"}
      </li>
      <li class="flex items-center gap-2">
        <span class="h-1.5 w-1.5 rounded-full" style="background: {matchCount > 0 ? 'var(--accent-green)' : 'var(--text-muted)'}"></span>
        {matchCount > 0 ? `${matchCount} matches imported` : "Import your match history (button above)"}
      </li>
      <li class="flex items-center gap-2">
        <span class="h-1.5 w-1.5 rounded-full" style="background: var(--text-muted)"></span>
        Play games -- Sentinel captures live timeline data automatically
      </li>
      <li class="flex items-center gap-2">
        <span class="h-1.5 w-1.5 rounded-full" style="background: var(--text-muted)"></span>
        Check Patterns and Improvement pages for insights
      </li>
    </ul>
  </div>

  {#if dbStats}
    <div class="mt-6 rounded-lg border p-4" style="background: var(--bg-tertiary); border-color: var(--border)">
      <p class="text-xs font-mono" style="color: var(--text-muted)">
        Sentinel v{appVersion} | DB: {dbStats.db_path}
      </p>
    </div>
  {/if}
</div>
