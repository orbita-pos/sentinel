<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { currentSummoner } from "../lib/stores/connection.js";
  import MatchCard from "../lib/components/MatchCard.svelte";
  import type { MatchSummaryItem } from "../lib/types/index.js";

  let matches: MatchSummaryItem[] = $state([]);
  let total = $state(0);
  let loading = $state(false);
  let backfillStatus: { current: number; total: number } | null = $state(null);
  let summoner = $derived($currentSummoner);

  async function loadMatches() {
    if (!summoner?.puuid) return;
    loading = true;
    try {
      const result = await invoke<{ matches: MatchSummaryItem[]; total: number }>(
        "get_match_history",
        { puuid: summoner.puuid, count: 50, offset: 0 }
      );
      matches = result.matches;
      total = result.total;
    } catch (e) {
      console.error("Failed to load matches:", e);
    }
    loading = false;
  }

  async function triggerBackfill() {
    if (!summoner?.puuid) return;
    backfillStatus = { current: 0, total: 0 };
    try {
      await invoke("trigger_backfill", { puuid: summoner.puuid });
    } catch (e) {
      console.error("Backfill failed:", e);
    }
    backfillStatus = null;
    await loadMatches();
  }

  // Listen for backfill progress
  listen<{ current: number; total: number }>("backfill-progress", (event) => {
    backfillStatus = event.payload;
  });
  listen("backfill-complete", () => {
    backfillStatus = null;
    loadMatches();
  });

  // Load matches when summoner changes
  $effect(() => {
    if (summoner?.puuid) {
      loadMatches();
    }
  });
</script>

<div class="mx-auto max-w-3xl">
  <div class="mb-6 flex items-center justify-between">
    <div>
      <h2 class="text-2xl font-bold" style="color: var(--text-primary)">Match History</h2>
      <p class="mt-1 text-sm" style="color: var(--text-secondary)">
        {total > 0 ? `${total} games analyzed` : "No games yet"}
      </p>
    </div>
    <button
      onclick={triggerBackfill}
      class="rounded-lg px-4 py-2 text-sm font-medium text-white transition-colors disabled:opacity-50"
      style="background: var(--accent-blue)"
      disabled={!summoner?.puuid || backfillStatus !== null}
    >
      {backfillStatus ? `Fetching ${backfillStatus.current}/${backfillStatus.total}...` : "Fetch Matches"}
    </button>
  </div>

  {#if !summoner?.puuid}
    <div class="flex h-64 items-center justify-center rounded-xl border" style="background: var(--bg-secondary); border-color: var(--border)">
      <div class="text-center">
        <p class="text-sm" style="color: var(--text-secondary)">Connect to League client to see match history</p>
        <p class="mt-1 text-xs" style="color: var(--text-muted)">Sentinel will sync your matches automatically when connected</p>
      </div>
    </div>
  {:else if loading}
    <div class="flex h-64 items-center justify-center">
      <p class="text-sm" style="color: var(--text-muted)">Loading matches...</p>
    </div>
  {:else if matches.length === 0}
    <div class="flex h-64 items-center justify-center rounded-xl border" style="background: var(--bg-secondary); border-color: var(--border)">
      <div class="text-center">
        <p class="text-sm" style="color: var(--text-secondary)">No matches found</p>
        <p class="mt-1 text-xs" style="color: var(--text-muted)">Click "Fetch Matches" to download your recent games</p>
      </div>
    </div>
  {:else}
    <div class="space-y-2">
      {#each matches as match_item (match_item.match_id)}
        <MatchCard match={match_item} />
      {/each}
    </div>
  {/if}
</div>
