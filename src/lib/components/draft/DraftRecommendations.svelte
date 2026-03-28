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

  // OP.GG matchup data for enemy champions
  let enemyMatchups: Record<string, any> = $state({});
  let matchupsLoading = $state(false);

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
        errorMsg = "No match data yet. Fetch matches first.";
      }
    } catch (e) {
      const msg = String(e);
      if (msg.includes("Not in champion select")) errorMsg = "Waiting for champion select...";
      else errorMsg = "Fetch matches first to get recommendations.";
      recommendations = [];
    }
    loading = false;
  }

  // Fetch OP.GG matchup data for enemy champions
  async function fetchEnemyMatchups() {
    if (!session) return;
    matchupsLoading = true;

    const enemies = session.their_team.filter(p => p.champion_id > 0);
    for (const enemy of enemies) {
      const champName = getChampionName(map, enemy.champion_id);
      if (!champName || champName.startsWith("Champion") || enemyMatchups[champName]) continue;

      try {
        // Get the champion key (Data Dragon key) for OP.GG
        const info = map[enemy.champion_id];
        if (!info?.key) continue;

        const data = await invoke<any>("get_champion_matchups", {
          champion: info.key,
          position: enemy.assigned_position?.toLowerCase() || "all",
        });
        if (data?.champion) {
          enemyMatchups[champName] = data;
          enemyMatchups = enemyMatchups; // trigger reactivity
        }
      } catch (e) {
        // Non-fatal, continue
      }
    }
    matchupsLoading = false;
  }

  // Auto-refresh when draft changes
  $effect(() => {
    if (!session || !summoner?.puuid) return;
    const bansKey = session.bans.join(",");
    const picksKey = [...session.my_team, ...session.their_team]
      .map(p => p.champion_id)
      .filter(id => id > 0)
      .join(",");
    const draftKey = `${bansKey}|${picksKey}`;

    if (draftKey !== lastDraftKey) {
      lastDraftKey = draftKey;
      refresh();
      fetchEnemyMatchups();
    }
  });

  $effect(() => {
    if (summoner?.puuid) refresh();
  });

  function scoreColor(score: number): string {
    if (score >= 70) return "var(--accent-green)";
    if (score >= 50) return "var(--accent-blue)";
    if (score >= 30) return "var(--accent-gold)";
    return "var(--text-muted)";
  }

  function tierColor(tier: string): string {
    if (tier === "S" || tier === "1") return "var(--accent-gold)";
    if (tier === "A" || tier === "2") return "var(--accent-green)";
    if (tier === "B" || tier === "3") return "var(--accent-blue)";
    return "var(--text-muted)";
  }

  // Compute which of your recommendations counter picked enemies
  let counterInsights = $derived(() => {
    if (!session || Object.keys(enemyMatchups).length === 0) return [];

    const insights: { message: string; type: "good" | "warn" }[] = [];

    // Check if any enemy champion has known counters that match your pool
    for (const [enemyChamp, data] of Object.entries(enemyMatchups)) {
      if (!data?.counters) continue;
      for (const counter of data.counters as any[]) {
        // Check if this counter is in your recommendations
        const inRecs = recommendations.some(r =>
          r.champion_name.toLowerCase() === counter.champion_name?.toLowerCase()
        );
        if (inRecs && counter.win_rate > 0.50) {
          insights.push({
            message: `${counter.champion_name} counters ${enemyChamp} (${(counter.win_rate * 100).toFixed(0)}% WR)`,
            type: "good",
          });
        }
      }
    }

    return insights.slice(0, 3);
  });
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

  <!-- Counter insights from OP.GG -->
  {#if counterInsights().length > 0}
    <div class="space-y-1">
      {#each counterInsights() as insight}
        <div class="rounded px-2 py-1.5 text-[10px]" style="background: rgba(34,197,94,0.08); border-left: 2px solid var(--accent-green); color: var(--accent-green)">
          {insight.message}
        </div>
      {/each}
    </div>
  {/if}

  <!-- Enemy matchup info -->
  {#if Object.keys(enemyMatchups).length > 0}
    <div class="rounded-lg px-2 py-1.5" style="background: var(--bg-tertiary)">
      <p class="text-[9px] font-bold uppercase mb-1" style="color: var(--text-muted)">Enemy Intel (OP.GG)</p>
      <div class="space-y-1">
        {#each Object.entries(enemyMatchups) as [champ, data]}
          {#if data?.win_rate}
            <div class="flex items-center gap-2 text-[10px]">
              <span class="font-medium" style="color: var(--accent-red)">{champ}</span>
              <span style="color: var(--text-muted)">{(data.win_rate * 100).toFixed(1)}% WR</span>
              {#if data.tier}
                <span class="rounded px-1 py-0.5 text-[8px] font-bold" style="color: {tierColor(data.tier)}">{data.tier}</span>
              {/if}
            </div>
          {/if}
        {/each}
      </div>
    </div>
  {/if}

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
