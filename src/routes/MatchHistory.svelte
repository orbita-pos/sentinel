<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { currentSummoner } from "../lib/stores/connection.js";
  import { currentRoute } from "../lib/stores/router.js";
  import MatchCard from "../lib/components/MatchCard.svelte";
  import PostGame from "./PostGame.svelte";
  import type { MatchSummaryItem } from "../lib/types/index.js";

  let matches: MatchSummaryItem[] = $state([]);
  let total = $state(0);
  let loading = $state(false);
  let backfillStatus: { current: number; total: number } | null = $state(null);
  let summoner = $derived($currentSummoner);

  // Stats derived from loaded matches
  let winRate = $derived(() => {
    if (matches.length === 0) return 0;
    return Math.round((matches.filter((m) => m.win).length / matches.length) * 100);
  });
  let avgKda = $derived(() => {
    if (matches.length === 0) return "0.0";
    const totalK = matches.reduce((s, m) => s + m.kills, 0);
    const totalD = matches.reduce((s, m) => s + m.deaths, 0);
    const totalA = matches.reduce((s, m) => s + m.assists, 0);
    return totalD === 0 ? "Perfect" : ((totalK + totalA) / totalD).toFixed(1);
  });
  let recentStreak = $derived(() => {
    if (matches.length === 0) return "";
    const first = matches[0].win;
    let count = 0;
    for (const m of matches) {
      if (m.win === first) count++;
      else break;
    }
    return `${count}${first ? "W" : "L"}`;
  });

  // Selected match for detail view
  let selectedMatchId: string | null = $state(null);

  async function loadMatches() {
    if (!summoner?.puuid) return;
    loading = true;
    try {
      const result = await invoke<{ matches: MatchSummaryItem[]; total: number }>(
        "get_match_history",
        { puuid: summoner.puuid, count: 100, offset: 0 }
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

  listen<{ current: number; total: number }>("backfill-progress", (event) => {
    backfillStatus = event.payload;
  });
  listen("backfill-complete", () => {
    backfillStatus = null;
    loadMatches();
  });
  listen("import-complete", () => {
    loadMatches();
  });

  $effect(() => {
    if (summoner?.puuid) {
      loadMatches();
    }
  });

  function openMatch(matchId: string) {
    selectedMatchId = matchId;
  }
  function closeDetail() {
    selectedMatchId = null;
  }
</script>

<div class="mx-auto max-w-4xl">
  {#if selectedMatchId}
    <!-- Match Detail View (inline) -->
    <div>
      <button
        onclick={closeDetail}
        class="mb-4 flex items-center gap-2 rounded-lg px-3 py-1.5 text-sm transition-colors"
        style="background: var(--bg-tertiary); color: var(--text-secondary)"
        onmouseenter={(e) => (e.currentTarget as HTMLElement).style.color = 'var(--text-primary)'}
        onmouseleave={(e) => (e.currentTarget as HTMLElement).style.color = 'var(--text-secondary)'}
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
        </svg>
        Back to Match History
      </button>
      <PostGame matchId={selectedMatchId} />
    </div>
  {:else}
    <!-- Match List View -->
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

    <!-- Quick Stats Bar -->
    {#if matches.length > 0}
      <div class="mb-4 flex gap-3">
        <div class="flex items-center gap-2 rounded-lg border px-4 py-2" style="background: var(--bg-secondary); border-color: var(--border)">
          <span class="text-xs" style="color: var(--text-muted)">Win Rate</span>
          <span class="text-sm font-bold" style="color: {winRate() >= 50 ? 'var(--accent-green)' : 'var(--accent-red)'}">
            {winRate()}%
          </span>
        </div>
        <div class="flex items-center gap-2 rounded-lg border px-4 py-2" style="background: var(--bg-secondary); border-color: var(--border)">
          <span class="text-xs" style="color: var(--text-muted)">Avg KDA</span>
          <span class="text-sm font-bold" style="color: var(--text-primary)">{avgKda()}</span>
        </div>
        <div class="flex items-center gap-2 rounded-lg border px-4 py-2" style="background: var(--bg-secondary); border-color: var(--border)">
          <span class="text-xs" style="color: var(--text-muted)">Streak</span>
          <span class="text-sm font-bold" style="color: {recentStreak().includes('W') ? 'var(--accent-green)' : 'var(--accent-red)'}">
            {recentStreak()}
          </span>
        </div>
        <div class="flex items-center gap-2 rounded-lg border px-4 py-2" style="background: var(--bg-secondary); border-color: var(--border)">
          <span class="text-xs" style="color: var(--text-muted)">Games</span>
          <span class="text-sm font-bold" style="color: var(--text-primary)">{total}</span>
        </div>
      </div>
    {/if}

    {#if !summoner?.puuid}
      <div class="flex h-48 items-center justify-center rounded-xl border" style="background: var(--bg-secondary); border-color: var(--border)">
        <div class="text-center">
          <p class="text-sm" style="color: var(--text-secondary)">Connect to League client to see match history</p>
          <p class="mt-1 text-xs" style="color: var(--text-muted)">Sentinel will sync your matches automatically when connected</p>
        </div>
      </div>
    {:else if loading}
      <div class="flex h-48 items-center justify-center">
        <p class="text-sm" style="color: var(--text-muted)">Loading matches...</p>
      </div>
    {:else if matches.length === 0}
      <div class="flex h-48 items-center justify-center rounded-xl border" style="background: var(--bg-secondary); border-color: var(--border)">
        <div class="text-center">
          <p class="text-sm" style="color: var(--text-secondary)">No matches found</p>
          <p class="mt-1 text-xs" style="color: var(--text-muted)">Click "Fetch Matches" or import from the Dashboard</p>
        </div>
      </div>
    {:else}
      <div class="space-y-2">
        {#each matches as match_item (match_item.match_id)}
          <MatchCard match={match_item} onclick={() => openMatch(match_item.match_id)} />
        {/each}
      </div>
    {/if}
  {/if}
</div>
