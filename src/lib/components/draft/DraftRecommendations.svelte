<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { currentSummoner } from "../../stores/connection.js";
  import { champSelectSession } from "../../stores/champselect.js";
  import { championMap, getChampionName, getChampionImageUrl, currentPatch } from "../../stores/champions.js";
  import type { DraftRecommendation } from "../../types/champselect.js";

  let recommendations: DraftRecommendation[] = $state([]);
  let loading = $state(false);
  let errorMsg = $state("");
  let summoner = $derived($currentSummoner);
  let session = $derived($champSelectSession);
  let map = $derived($championMap);
  let patch = $derived($currentPatch);

  // Track bans+picks to detect changes and auto-refresh
  let lastDraftKey = $state("");

  async function refresh() {
    if (!summoner?.puuid) return;
    loading = true;
    errorMsg = "";
    try {
      recommendations = await invoke<DraftRecommendation[]>("get_draft_recommendations", {
        puuid: summoner.puuid,
      });
      if (recommendations.length === 0) {
        errorMsg = "No match data yet. Go to Match History and click 'Fetch Matches' first.";
      }
    } catch (e) {
      const msg = String(e);
      if (msg.includes("Not in champion select")) {
        errorMsg = "Waiting for champion select...";
      } else {
        errorMsg = "Fetch matches first to get recommendations.";
      }
      recommendations = [];
    }
    loading = false;
  }

  // Auto-refresh when draft state changes (new bans/picks)
  $effect(() => {
    if (!session || !summoner?.puuid) return;
    const bansKey = session.bans.join(",");
    const picksKey = [...session.my_team, ...session.their_team]
      .map((p) => p.champion_id)
      .filter((id) => id > 0)
      .join(",");
    const draftKey = `${bansKey}|${picksKey}`;

    if (draftKey !== lastDraftKey) {
      lastDraftKey = draftKey;
      refresh();
    }
  });

  // Also refresh on initial mount
  $effect(() => {
    if (summoner?.puuid) {
      refresh();
    }
  });

  function scoreColor(score: number): string {
    if (score >= 70) return "var(--accent-green)";
    if (score >= 50) return "var(--accent-blue)";
    if (score >= 30) return "var(--accent-gold)";
    return "var(--text-muted)";
  }
</script>

<div class="flex flex-col gap-2">
  <div class="flex items-center justify-between">
    <h3 class="text-xs font-semibold uppercase tracking-wide" style="color: var(--accent-purple)">
      Recommendations
    </h3>
    <button
      onclick={refresh}
      class="rounded px-2 py-0.5 text-[10px] transition-colors"
      style="background: var(--bg-tertiary); color: var(--text-secondary)"
      disabled={loading}
    >
      {loading ? "..." : "Refresh"}
    </button>
  </div>

  {#if errorMsg && recommendations.length === 0}
    <div class="rounded-lg border px-3 py-4 text-center text-xs" style="background: var(--bg-tertiary); border-color: var(--border); color: var(--text-muted)">
      {errorMsg}
    </div>
  {:else}
    {#each recommendations as rec}
      {@const img = getChampionImageUrl(map, rec.champion_id, patch)}
      <div class="rounded-lg border px-3 py-2.5" style="background: var(--bg-tertiary); border-color: var(--border)">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            {#if img}
              <img src={img} alt={rec.champion_name} class="h-6 w-6 rounded" />
            {/if}
            <span class="text-sm font-semibold" style="color: var(--text-primary)">
              {rec.champion_name}
            </span>
            <span class="rounded px-1.5 py-0.5 text-[10px] font-bold" style="background: var(--bg-primary); color: {scoreColor(rec.score)}">
              {rec.score.toFixed(0)}
            </span>
          </div>
          <span class="text-xs" style="color: {rec.personal_wr >= 0.5 ? 'var(--accent-green)' : 'var(--accent-red)'}">
            {(rec.personal_wr * 100).toFixed(0)}% WR
          </span>
        </div>
        <div class="mt-1 flex flex-wrap gap-1">
          {#each rec.reasons as reason}
            <span class="text-[10px]" style="color: var(--text-muted)">{reason}</span>
            {#if rec.reasons.indexOf(reason) < rec.reasons.length - 1}
              <span class="text-[10px]" style="color: var(--text-muted)">-</span>
            {/if}
          {/each}
        </div>
      </div>
    {/each}
  {/if}
</div>
